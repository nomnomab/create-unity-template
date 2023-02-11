use serde_derive::Deserialize;
use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    process::exit,
};

#[derive(Deserialize)]
pub struct Config {
    pub essentials: Essentials,
}

impl Config {
    pub fn _get_package_folder(&self, version: &Version) -> String {
        format!(
            "{}\\{}.{}\\Editor\\Data\\Resources\\PackageManager\\",
            self.essentials.unity_hub_path, version.major, version.minor
        )
    }

    pub fn get_template_folder(&self, version: &str) -> String {
        format!(
            "{}\\{}\\Editor\\Data\\Resources\\PackageManager\\ProjectTemplates\\",
            self.essentials.unity_hub_path, version
        )
    }

    pub fn _get_built_in_packages_folder(&self, version: &Version) -> String {
        format!(
            "{}\\{}.{}\\Editor\\Data\\Resources\\PackageManager\\BuiltInPackages\\",
            self.essentials.unity_hub_path, version.major, version.minor
        )
    }

    pub fn get_editor_folder(&self, version: &Version) -> String {
        format!(
            "{}\\{}.{}\\Editor\\Data\\Resources\\PackageManager\\Editor\\",
            self.essentials.unity_hub_path, version.major, version.minor
        )
    }
}

#[derive(Deserialize)]
pub struct Essentials {
    pub unity_hub_path: String,
    pub default_dependencies: Vec<String>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Version {
    pub major: String,
    pub minor: String,
}

impl Version {
    pub fn parse(dir: &DirEntry) -> Version {
        let name = dir.file_name();
        let name = &name.to_str().unwrap();
        let last_index = name.rfind('.').unwrap();
        let major = name[..last_index].to_string();
        let minor = name[(last_index + 1)..].to_string();

        Version { major, minor }
    }
}

#[derive(Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct BuiltInPackage {
    pub name: Option<String>,
    pub version: Option<String>,
}

pub fn load_config() -> Config {
    let file_name = "config.toml";
    let contents = match fs::read_to_string(file_name) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("Could not read config file at `{}`", file_name);

            // make a new one
            let default_toml = r#"
[essentials]
# path to the unity hub editor folder
unity_hub_path="C:\\Program Files\\Unity\\Hub\\Editor"
# all default dependencies to select
default_dependencies=[
    "com.unity.collab-proxy",
    "com.unity.feature.development",
    "com.unity.textmeshpro",
    "com.unity.timeline",
    "com.unity.visualscripting",
    "com.unity.ugui",
]
            "#
            .trim_start();

            fs::write(file_name, default_toml)
                .unwrap_or_else(|_| panic!("Failed to write default config file"));

            println!(
                "- New config created, please open it and modify it with correct information."
            );

            exit(1);
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Could not load config from `{}`", file_name);
            eprintln!("> {}", e);
            exit(1);
        }
    };

    config
}

pub fn load_versions(config: &Config) -> Vec<Version> {
    let path = &config.essentials.unity_hub_path;
    let dirs = match fs::read_dir(path) {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("Could not load directories from `{}`", path);
            eprintln!("> {}", e);
            exit(1);
        }
    };

    let mut versions = HashSet::new();
    dirs.map(|dir| Version::parse(&dir.unwrap()))
        .filter(|version| versions.insert(version.clone()))
        .collect()
}

pub fn load_dependencies_from(path: &str) -> Vec<BuiltInPackage> {
    let path = format!("{}\\Packages\\manifest.json", path);
    let package_file = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Could not load manifest.json from `{}`", path);
            eprintln!("> {}", e);
            exit(1);
        }
    };

    let package_data: serde_json::Value = match serde_json::from_str(&package_file) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Could not load manifest.json from `{}`", path);
            eprintln!("> {}", e);
            exit(1);
        }
    };

    let deps = &package_data["dependencies"];
    let deps = deps
        .as_object()
        .or_else(|| panic!("Could not load [dependencies] from manifest.json"))
        .unwrap();

    deps.iter()
        .map(|dep| BuiltInPackage {
            name: Some(dep.0.to_owned().to_string()),
            version: Some(dep.1.to_owned().to_string()),
        })
        .collect()
}

pub fn load_dependencies(config: &Config, version: &Version) -> Vec<BuiltInPackage> {
    let path = &config.get_editor_folder(version);
    let files = match fs::read_dir(path) {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("Could not load directories from `{}`", path);
            eprintln!("> {}", e);
            exit(1);
        }
    };

    files
        .map(|file| {
            let dir = match file {
                Ok(file) => file,
                Err(_) => return None,
            };

            let path = dir.file_name();
            let file_name = path.to_str().unwrap();
            let dash_index = match file_name.rfind("-") {
                Some(index) => index,
                None => return None,
            };
            let name = file_name[..dash_index].to_string();
            let version = file_name[(dash_index + 1)..].to_string();

            Some(BuiltInPackage {
                name: Some(name),
                version: Some(version),
            })
        })
        .filter(|file| file.is_some())
        .map(|file| file.unwrap())
        .collect()
}
