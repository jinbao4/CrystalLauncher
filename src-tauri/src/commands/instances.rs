use crate::models::fs::LauncherPaths;
use crate::models::instance::{Instance, InstallState};
use std::sync::Mutex;
use tauri::State;

fn load_instance(instances_dir: &std::path::Path, version: &str) -> Option<Instance> {
    let path = instances_dir.join(version).join("instance.json");
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn save_instance(instances_dir: &std::path::Path, instance: &Instance) -> Result<(), String> {
    let dir = instances_dir.join(&instance.mc_version);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create instance dir: {}", e))?;

    let json = serde_json::to_string_pretty(instance)
        .map_err(|e| format!("Failed to serialize instance: {}", e))?;

    std::fs::write(dir.join("instance.json"), json)
        .map_err(|e| format!("Failed to write instance.json: {}", e))?;

    Ok(())
}

pub fn get_or_create_instance(
    instances_dir: &std::path::Path,
    version: &str,
) -> Result<Instance, String> {
    if let Some(existing) = load_instance(instances_dir, version) {
        return Ok(existing);
    }

    let instance = Instance::new(version);
    save_instance(instances_dir, &instance)?;
    Ok(instance)
}

pub fn persist_instance(
    instances_dir: &std::path::Path,
    instance: &Instance,
) -> Result<(), String> {
    save_instance(instances_dir, instance)
}

#[tauri::command]
pub fn list_instances(
    paths: State<Mutex<LauncherPaths>>,
) -> Result<Vec<Instance>, String> {
    let launcher_paths = paths
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    let instances_dir = &launcher_paths.instances;
    std::fs::create_dir_all(instances_dir)
        .map_err(|e| format!("Failed to create instances dir: {}", e))?;

    let mut instances: Vec<Instance> = std::fs::read_dir(instances_dir)
        .map_err(|e| format!("Failed to read instances dir: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.path().is_dir() {
                return None;
            }
            let version = entry.file_name().into_string().ok()?;
            load_instance(instances_dir, &version)
        })
        .collect();
    instances.sort_by(|a, b| a.mc_version.cmp(&b.mc_version));

    Ok(instances)
}

#[tauri::command]
pub fn ensure_instance(
    version: String,
    paths: State<Mutex<LauncherPaths>>,
) -> Result<Instance, String> {
    let launcher_paths = paths
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;

    get_or_create_instance(&launcher_paths.instances, &version)
}