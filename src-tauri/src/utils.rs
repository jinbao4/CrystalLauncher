use std::path::Path;
use rayon::prelude::*;
use crate::models::mc::{VersionManifest, AssetMap, Rule};

pub fn collect_jars(dir: &Path, jars: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_jars(&path, jars);
            } else if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                jars.push(path.to_string_lossy().into_owned());
            }
        }
    }
}

pub fn get_classpath_separator() -> &'static str {
    if cfg!(target_os = "windows") { ";" } else { ":" }
}

pub fn download_file_if_needed(url: &str, path: &Path) -> Result<(), String> {
    if path.exists() {
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.len() > 0 { return Ok(()); } 
        }
    }
    
    let bytes = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, &bytes).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn download_client_jar(instance_dir: &Path, version_json: &str) -> Result<(), String> {
    let manifest: VersionManifest = serde_json::from_str(version_json).map_err(|e| e.to_string())?;
    download_file_if_needed(&manifest.downloads.client.url, &instance_dir.join("client.jar"))
}

pub fn download_assets(assets_root: &Path, version_json: &str) -> Result<(), String> {
    let manifest: VersionManifest = serde_json::from_str(version_json).map_err(|e| e.to_string())?;
    let objects_dir = assets_root.join("objects");
    
    let index_url = &manifest.asset_index.url;
    let index_path = assets_root.join("indexes").join(format!("{}.json", manifest.asset_index.id));
    download_file_if_needed(index_url, &index_path)?;

    let index_json = std::fs::read_to_string(index_path).map_err(|e| e.to_string())?;
    let asset_map: AssetMap = serde_json::from_str(&index_json).map_err(|e| e.to_string())?;
    
    asset_map.objects.par_iter().for_each(|(_, object)| {
        let hash_prefix = &object.hash[0..2];
        let path = objects_dir.join(hash_prefix).join(&object.hash);
        let url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, object.hash);
        let _ = download_file_if_needed(&url, &path);
    });
    Ok(())
}

pub fn is_library_allowed(rules: &Option<Vec<Rule>>) -> bool {
    let Some(rules_list) = rules else { return true; };
    let mut allowed = false;
    for rule in rules_list {
        let os_applies = if let Some(os) = &rule.os {
            (os.name == "windows" && cfg!(target_os = "windows")) ||
            (os.name == "osx" && cfg!(target_os = "macos")) ||
            (os.name == "linux" && cfg!(target_os = "linux"))
        } else { true };
        if os_applies { allowed = rule.action == "allow"; }
    }
    allowed
}

fn get_os_key() -> &'static str {
    if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "macos") { "osx" } // Mojang uses "osx" for macOS
    else { "linux" }
}

pub fn download_libraries(libraries_root: &Path, version_json: &str) -> Result<(), String> {
    let manifest: VersionManifest = serde_json::from_str(version_json).map_err(|e| e.to_string())?;
    
    manifest.libraries.par_iter().for_each(|lib| {
        if is_library_allowed(&lib.rules) {
            if let Some(artifact) = &lib.downloads.artifact {
                let _ = download_file_if_needed(&artifact.url, &libraries_root.join(&artifact.path));
            }

            if let Some(natives_map) = &lib.natives {
                let os_key = get_os_key();
                
                if let Some(classifier_key) = natives_map.get(os_key) {
                    if let Some(classifiers) = &lib.downloads.classifiers {
                        if let Some(artifact) = classifiers.get(classifier_key) {
                             let _ = download_file_if_needed(&artifact.url, &libraries_root.join(&artifact.path));
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

pub fn extract_natives(
    libraries_root: &Path, 
    natives_dir: &Path, 
    version_manifest: &VersionManifest
) -> Result<(), String> {
    if natives_dir.exists() {
        std::fs::remove_dir_all(natives_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(natives_dir).map_err(|e| e.to_string())?;

    for lib in &version_manifest.libraries {
        if is_library_allowed(&lib.rules) {
            if let Some(natives_map) = &lib.natives {
                let os_key = get_os_key();
                
                if let Some(classifier_key) = natives_map.get(os_key) {
                    if let Some(classifiers) = &lib.downloads.classifiers {
                        if let Some(artifact) = classifiers.get(classifier_key) {
                            let jar_path = libraries_root.join(&artifact.path);
                            
                            // Unzip the native jar
                            if let Ok(file) = std::fs::File::open(&jar_path) {
                                let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                                
                                for i in 0..archive.len() {
                                    let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
                                    let path = file.mangled_name();
                                    
                                    if path.starts_with("META-INF") || file.is_dir() { continue; }
                                    
                                    let out_path = natives_dir.join(file.name());
                                    let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                                    std::io::copy(&mut file, &mut out_file).map_err(|e| e.to_string())?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}