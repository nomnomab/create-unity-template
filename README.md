# create-unity-template

A CLI tool to easily create custom unity project templates.

![](/assets/template.png)

## Building the tool

```rs
cargo build --release
```

## Config file

The default config file looks like this:

```toml
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
```

## Creating a new template

![](/assets/new.png)

> The name will be autoformatted for you to comply with the template naming requirements

```rs
create-unity-template.exe new template-name
```

The creation process will go through multiple steps:

1. Display name
   - The shown name in the Unity Hub
2. Description
   - The text to show in the Unity Hub when selected
3. Keywords
   - Used for filtering
   - Needs to be in comma-separated format
   - Not entire sure if this is used outside of packages
4. Unity version
   - The Unity version the creator will operate upon
   - Will be shown in a selection list
5. Package version
   - The version of this package template
6. Project path
   - The path to the project this template will use to clone
   - Point to the root folder
7. Dependencies
   - The build-in dependencies that will be included in the manifest
   - Will be shown in a selection list
   - The default dependencies from the config will be auto-selected

The built template will be in the `/builds/` folder. There it can be modified to include whatever you need, such as additional specific dependencies.

## Packing a template

![](/assets/pack.png)

```rs
create-unity-template.exe pack
```

> The template will be packed with a single-ran script when the project is loaded. This will automatically replace the generated manifest file with the one you created with the tool.

After packing, the `.tgz` file will be located in the `/outputs/` folder. This file will need to be copied into the package folder associated with the version you built for (will be shown in the terminal after packing). After that, restart the Unity Hub to have it refresh its template cache.
