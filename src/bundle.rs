use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Dependencies {}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub name: String,
    #[serde(rename(serialize = "displayName"))]
    pub display_name: String,
    pub version: String,
    pub unity: String,
    #[serde(rename(serialize = "unityFull"))]
    pub unity_full: String,
    pub keywords: Vec<String>,
    pub category: String,
    pub description: String,
    pub dependencies: serde_json::Value,
}

pub fn build(project_path: &str, data: Data) {
    // 1. build a tar file with all contents
    // 2. build a tar.gz file
    // 3. map to a tgz file

    // com.company.template.name-version.tgz
    // - com.company.template.name-version.tar
    // - - package/
    // - - - Documentation~/
    // - - - ProjectData~/
    // - - - - Assets/
    // - - - - Packages/
    // - - - - - manifest.json
    // - - - - ProjectSettings/
    // - - - Tests/
    // - - - CHANGELOG.md
    // - - - LICENSE.md
    // - - - package.json
    // - - - README.md

    let folders = [
        "package",
        "package\\Documentation~",
        "package\\ProjectData~",
        "package\\ProjectData~\\Assets",
        "package\\ProjectData~\\Packages",
        "package\\ProjectData~\\ProjectSettings",
        "package\\Tests",
    ];

    let files = [
        "package\\ProjectData~\\Packages\\manifest.json",
        "package\\CHANGELOG.md",
        "package\\LICENSE.md",
        "package\\package.json",
        "package\\README.md",
    ];

    let path = format!(
        ".\\builds\\com.unity.template.{}-{}\\",
        data.name, data.version
    );

    if let Ok(_) = fs::read_dir(&path) {
        fs::remove_dir_all(&path)
            .unwrap_or_else(|_| panic!("Failed to remove build directory at {}", path));
    }

    fs::create_dir_all(&path)
        .unwrap_or_else(|_| panic!("Failed to create build directory at {}", path));
    let root_dir = Path::new(&path);

    // make folders
    for folder in folders.map(|dir| root_dir.join(dir)) {
        let _ = fs::create_dir_all(&folder)
            .unwrap_or_else(|_| panic!("Failed to create build directory at {:?}", folder));
    }

    // make files
    for file in files.map(|file| root_dir.join(file)) {
        let extension = file
            .extension()
            .unwrap_or_else(|| panic!("Failed to get file extension at {:?}", file));

        let contents = match extension.to_str().unwrap() {
            "md" => Some(String::new()),
            "json" => match file.file_name() {
                Some(name) => match name.to_str().unwrap() {
                    "package.json" => Some(serde_json::to_string_pretty(&data).unwrap()),
                    _ => Some("{}".into()),
                },
                None => None,
            },
            _ => None,
        };

        if let Some(contents) = contents {
            touch(&file).unwrap_or_else(|_| panic!("Failed to write to file at {:?}", file));

            fs::write(&file, contents)
                .unwrap_or_else(|_| panic!("Failed to write to file at {:?}", file));
        } else {
            panic!("Failed to write file to {:?}", file);
        }
    }

    // copy project to dir
    let folders = ["Assets", "Packages", "ProjectSettings"];

    let project_dir = Path::new(project_path);
    let project_name = project_dir
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    for project_folder in folders.map(|dir| project_dir.join(dir)) {
        let project_folder_str = project_folder.to_str().unwrap();
        let project_folder_start: String = project_folder_str
            .split('\\')
            .skip_while(|dir| dir != &project_name)
            .map(|dir| dir.to_string() + "\\")
            .collect();
        let project_folder_start = &project_folder_start[project_name.len()..];
        let build_path = root_dir.join(format!("package\\ProjectData~{}", project_folder_start));

        copy_dir_all(project_folder, build_path).unwrap_or_else(|v| panic!("Failed to copy {}", v));
    }

    // clean up
    let files = [
        "package\\ProjectData~\\Packages\\packages-lock.json",
        "package\\ProjectData~\\ProjectSettings\\ProjectVersion.txt",
    ];

    for file in files.map(|file| root_dir.join(file)) {
        fs::remove_file(&file).unwrap_or_else(|_| panic!("Failed to remove file at {:?}", file));
    }

    // copy deps into manifest.json

    #[derive(Serialize)]
    struct Dependencies {
        dependencies: serde_json::Value,
    }

    let deps = serde_json::to_string_pretty(&Dependencies {
        dependencies: data.dependencies.clone(),
    })
    .unwrap();

    let manifest_path = root_dir.join("package\\ProjectData~\\Packages\\manifest.json");
    fs::write(&manifest_path, &deps)
        .unwrap_or_else(|_| panic!("Failed to write manifest file at {:?}", manifest_path));
    let manifest_path = root_dir.join("package\\ProjectData~\\Assets\\manifest.json");
    fs::write(&manifest_path, &deps)
        .unwrap_or_else(|_| panic!("Failed to write manifest file at {:?}", manifest_path));

    println!();
    println!("Build folder is located at:");
    println!("- {}", root_dir.to_str().unwrap());
    println!();
    println!("To pack the template, run:");
    println!("- cargo run pack");
    println!();

    // open root folder
    // TODO: Handle Mac and Linux
    // https://stackoverflow.com/questions/66485945/with-rust-open-explorer-on-a-file
    // Command::new("explorer").arg(path).spawn().unwrap();
}

// A simple implementation of `% touch path` (ignores existing files)
fn touch(path: &Path) -> std::io::Result<()> {
    match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
