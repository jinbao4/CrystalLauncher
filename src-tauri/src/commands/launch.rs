use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::path::PathBuf;
use tauri::State;
use crate::models::fs::LauncherPaths;
use crate::models::mc::VersionManifest;
use crate::utils;

#[tauri::command]
pub fn launch_instance(
    instance_name: String, 
    uuid: String,          
    name: String,          
    access_token: String,  
    paths: State<'_, Mutex<LauncherPaths>>
) -> Result<(), String> {
    let p = paths.lock().map_err(|e| e.to_string())?;
    
    let instance_dir = p.instances.join(&instance_name);
    let libraries_root = p.root.join("libraries");
    let assets_root = p.root.join("assets");
    let client_jar = instance_dir.join("client.jar");
    let version_file = instance_dir.join("version.json");
    
    let natives_dir = instance_dir.join("natives"); 

    if !version_file.exists() {
        return Err(format!("version.json missing for {}. Did you install it?", instance_name));
    }
    
    let version_text = std::fs::read_to_string(&version_file)
        .map_err(|e| format!("Failed to read version.json: {}", e))?;
        
    let manifest: VersionManifest = serde_json::from_str(&version_text)
        .map_err(|e| format!("Invalid version.json: {}", e))?;

    let official_mc = LauncherPaths::official_mc(); 

    utils::extract_natives(&libraries_root, &natives_dir, &manifest)?;

    let mut jar_list = Vec::new();
    
    for lib in &manifest.libraries {
        if utils::is_library_allowed(&lib.rules) {
            if let Some(artifact) = &lib.downloads.artifact {
                let lib_path = libraries_root.join(&artifact.path);
                jar_list.push(lib_path.to_string_lossy().into_owned());
            }
        }
    }

    jar_list.push(client_jar.to_string_lossy().into_owned());

    for jar in &jar_list {
        if !PathBuf::from(jar).exists() {
             return Err(format!("Missing library: {}", jar));
        }
    }
    
    let separator = utils::get_classpath_separator();
    let classpath = jar_list.join(separator);

    println!("Launching {} on {}", instance_name, std::env::consts::OS);

    Command::new("java")
        .arg("-Xmx4G")             
        .arg(format!("-Djava.library.path={}", natives_dir.to_string_lossy()))
        .arg("-cp").arg(classpath)
        .arg("net.minecraft.client.main.Main")
        .arg("--version").arg(&manifest.id)
        .arg("--accessToken").arg(&access_token)
        .arg("--uuid").arg(&uuid)
        .arg("--username").arg(&name)
        .arg("--userType").arg("msa")
        .arg("--assetsDir").arg(assets_root)
        .arg("--assetIndex").arg(&manifest.asset_index.id) 
        .arg("--gameDir").arg(official_mc) 
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Launch failed: {}. Is Java installed?", e))?;

    Ok(())
}