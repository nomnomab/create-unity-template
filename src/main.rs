mod bundle;
mod config;

use std::{
    fs::{DirEntry, File},
    process::exit,
};

use clap::{Parser, Subcommand};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use flate2::{write::GzEncoder, Compression};
use serde_json::{json, Map};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    pub basic_commands: BasicCommands,
}

#[derive(Debug, Subcommand)]
enum BasicCommands {
    /// Creates a new unity template
    New(NewCommand),

    /// Packs a unity template from a generated build
    Pack(PackCommand),
}

#[derive(Debug, clap::Args)]
pub struct NewCommand {
    pub name: String,
}

#[derive(Debug, clap::Args)]
pub struct PackCommand {
    // pub name: String,
}

fn main() {
    let config = config::load_config();

    println!("[create-unity-template - Created by Andrew Burke]");
    let args = Args::parse();

    match args.basic_commands {
        BasicCommands::New(cmd) => create_project(config, cmd).unwrap(),
        BasicCommands::Pack(cmd) => pack_project(config, cmd).unwrap(),
    };
}

fn create_project(config: config::Config, cmd: NewCommand) -> std::io::Result<()> {
    let versions = config::load_versions(&config);

    // name
    // displayName
    // version
    // unity
    // description
    // dependencies
    // keywords
    // category

    let mut name = cmd.name.to_lowercase().replace("-", "");
    name.retain(|c| !c.is_whitespace());

    // displayName
    // QOL parse the name as the display name
    let display_name: String = cmd
        .name
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .fold(String::new(), |acc, w| format!("{} {}", acc, w));

    // displayName
    let display_name = Input::<String>::new()
        .with_prompt("Display name")
        .with_initial_text(display_name.trim())
        .interact_text()?;

    // description
    let description = Input::<String>::new()
        .with_prompt("Description")
        .with_initial_text("")
        .allow_empty(true)
        .interact_text()?;

    // keywords
    let keywords = Input::<String>::new()
        .with_prompt("Keywords")
        .with_initial_text("")
        .allow_empty(true)
        .interact_text()?;

    // category
    let items = vec!["2D", "3D"];
    let category = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .with_prompt("Category")
        .interact_on_opt(&Term::stderr())?;

    let category = match category {
        Some(index) => items[index].clone(),
        None => {
            eprintln!("Did not select a category.");
            exit(1);
        }
    };

    // unity
    let items: Vec<String> = versions
        .iter()
        .map(|version| format!("{}.{}", version.major, version.minor))
        .collect();

    let unity = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .with_prompt("Unity version")
        .interact_on_opt(&Term::stderr())?;

    let unity = match unity {
        Some(index) => versions[index].clone(),
        None => {
            eprintln!("Did not select a version.");
            exit(1);
        }
    };

    // version
    let version = Input::<String>::new()
        .with_prompt("Package version")
        .with_initial_text("0.0.1")
        .interact_text()?;

    // projectPath
    let project_path = Input::<String>::new()
        .with_prompt("Project path")
        .interact_text()?;

    // dependencies
    let mut project_deps = config::load_dependencies_from(&project_path);
    let mut built_in_deps = config::load_dependencies(&config, &unity);
    built_in_deps.append(&mut project_deps);
    built_in_deps.sort();
    built_in_deps.dedup_by(|a, b| a.name.eq(&b.name));

    let items: Vec<String> = built_in_deps
        .iter()
        .map(|dep| dep.name.clone().unwrap())
        .collect();
    let items_defaults: Vec<bool> = items
        .iter()
        .map(|item| config.essentials.default_dependencies.contains(item))
        .collect();

    let dependencies = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&items)
        .with_prompt("Dependencies")
        .defaults(&items_defaults)
        .max_length(10)
        .interact_on_opt(&Term::stderr())?;

    let dependencies = {
        let mut buffer = Vec::new();

        if let Some(indices) = dependencies {
            for index in indices {
                buffer.push(built_in_deps.get(index));
            }
        }

        buffer
    };

    let dependencies = {
        let mut map: Map<String, serde_json::Value> = Map::new();

        for dep in dependencies.iter().map(|dep| dep.unwrap()) {
            let version = dep.version.clone().unwrap().to_string();
            map.insert(
                dep.name.clone().unwrap(),
                serde_json::Value::String(version[1..(version.len() - 1)].to_string()),
            );
        }

        map
    };

    let data = bundle::Data {
        name,
        display_name,
        version,
        unity: unity.major.clone(),
        unity_full: format!("{}.{}", unity.major, unity.minor),
        keywords: keywords
            .split(',')
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty())
            .collect(),
        category: category.to_string(),
        description,
        dependencies: json!(dependencies),
    };

    bundle::build(&project_path, data);

    Ok(())
}

#[allow(unused_must_use)]
fn pack_project(config: config::Config, _: PackCommand) -> std::io::Result<()> {
    // load up all builds in the folder in a list
    let build_path = ".\\builds";
    let dir = fs::read_dir(&build_path)?;

    let builds: Vec<DirEntry> = dir
        .filter(|path| path.as_ref().unwrap().path().is_dir())
        .map(|path| path.unwrap())
        .collect();

    let items: Vec<String> = builds
        .iter()
        .map(|dir| dir.file_name().to_str().unwrap().to_string())
        .collect();
    let project = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .with_prompt("Project to build")
        .interact_on_opt(&Term::stderr())?;

    let project = match project {
        Some(index) => &builds[index],
        None => {
            eprintln!("Did not select a project.");
            exit(1);
        }
    };

    let project_name = project.file_name();

    // package folder insides
    let mut project = project.path().read_dir().unwrap();
    let output_path = ".\\outputs";

    fs::create_dir(output_path);

    let path = format!("{}\\{}.tgz", output_path, project_name.to_str().unwrap());
    let tar_gz = File::create(&path).unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());

    let mut tar = tar::Builder::new(enc);
    let project_path = project.next().unwrap().unwrap().path();
    let project_path = project_path.to_str().unwrap();
    tar.append_dir_all("package", &project_path)
        .unwrap_or_else(|e| panic!("Failed to pack tar file: {:?}", e));

    let contents = r#"
using System.IO;
using UnityEditor;
using UnityEngine;

public static class ___ManifestOverride
{
    private static readonly string InputPath = $"{Application.dataPath}/manifest.json";

    [InitializeOnLoadMethod]
    private static void OnLoad()
    {
        if (!File.Exists(InputPath))
        {
            Debug.LogWarning($"Input path does not exist at: {InputPath}");
            Debug.LogWarning("___ManifestOverride.cs is most likely completed. Remove this file if so.");
            return;
        }

        var targetPath = Path.Join(Application.dataPath, "..\\Packages\\manifest.json");

        File.Copy(InputPath, targetPath, true);

        // delete this file
        AssetDatabase.DeleteAsset("Assets/___ManifestOverride.cs");
        AssetDatabase.DeleteAsset("Assets/manifest.json");
        AssetDatabase.SaveAssets();
        AssetDatabase.Refresh();
    }
}
    "#.trim_start();

    std::fs::write(".\\___ManifestOverride.cs", &contents)
        .unwrap_or_else(|e| panic!("Failed to create packer class file: {:?}", e));
    let mut override_file = File::open(".\\___ManifestOverride.cs").unwrap();
    tar.append_file(
        "package\\ProjectData~\\Assets\\___ManifestOverride.cs",
        &mut override_file,
    )
    .unwrap_or_else(|e| panic!("Failed to pack tar file: {:?}", e));
    std::fs::remove_file(".\\___ManifestOverride.cs")
        .unwrap_or_else(|e| panic!("Failed to delete packer class file: {:?}", e));

    // load package.json from project
    let package_json_path = format!("{}\\package.json", &project_path);
    println!("{}", package_json_path);

    let contents = fs::read_to_string(&package_json_path).unwrap();
    let data: serde_json::Value = serde_json::from_str(&contents).unwrap();
    let version = &data.as_object().unwrap()["unityFull"].as_str().unwrap();

    let template_folder = config.get_template_folder(version);

    println!();
    println!("Output .tgz is located at:");
    println!("- {}", path);
    println!();
    println!("Copy the .tgz file into:");
    println!("- {}", template_folder);
    println!();
    println!("After copying, completely restart the Unity Hub.");
    println!();

    Ok(())
}
