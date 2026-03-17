use crate::models::mc::{AssetMap, Rule, VersionManifest};
use std::path::{Path, PathBuf};

#[allow(dead_code)]
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
    if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }
}

pub fn download_file_if_needed(url: &str, path: &Path) -> Result<(), String> {
    if path.exists() {
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.len() > 0 {
                return Ok(());
            }
        }
    }

    let client = reqwest::blocking::Client::new();
    let bytes = client.get(url)
        .send()
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
    let manifest: VersionManifest =
        serde_json::from_str(version_json).map_err(|e| e.to_string())?;
    download_file_if_needed(
        &manifest.downloads.client.url,
        &instance_dir.join("client.jar"),
    )
}

pub fn download_assets(assets_root: &Path, version_json: &str) -> Result<(), String> {
    let manifest: VersionManifest =
        serde_json::from_str(version_json).map_err(|e| e.to_string())?;
    let objects_dir = assets_root.join("objects");

    let index_url = &manifest.asset_index.url;
    let index_path = assets_root
        .join("indexes")
        .join(format!("{}.json", manifest.asset_index.id));
    download_file_if_needed(index_url, &index_path)?;

    let index_json = std::fs::read_to_string(index_path).map_err(|e| e.to_string())?;
    let asset_map: AssetMap = serde_json::from_str(&index_json).map_err(|e| e.to_string())?;

    for (_, object) in asset_map.objects {
        let hash_prefix = &object.hash[0..2];
        let path = objects_dir.join(hash_prefix).join(&object.hash);
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            hash_prefix, object.hash
        );
        let _ = download_file_if_needed(&url, &path);
    }
    Ok(())
}

pub fn is_library_allowed(rules: &Option<Vec<Rule>>) -> bool {
    let Some(rules_list) = rules else {
        return true;
    };
    let mut allowed = false;
    for rule in rules_list {
        let os_applies = if let Some(os) = &rule.os {
            (os.name == "windows" && cfg!(target_os = "windows"))
                || (os.name == "osx" && cfg!(target_os = "macos"))
                || (os.name == "linux" && cfg!(target_os = "linux"))
        } else {
            true
        };
        if os_applies {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn get_os_key() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    }
    // Mojang uses "osx" for macOS
    else {
        "linux"
    }
}

pub fn get_adoptium_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    }
}

pub fn get_adoptium_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x64" // default
    }
}

#[derive(serde::Deserialize)]
struct AdoptiumAsset {
    binary: AdoptiumBinary,
}

#[derive(serde::Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
}

#[derive(serde::Deserialize)]
struct AdoptiumPackage {
    link: String,
}

pub fn download_jre(jres_root: &Path, major_version: u32) -> Result<PathBuf, String> {
    let jre_dir = jres_root.join(format!("jre{}", major_version));
    if jre_dir.exists() {
        return Ok(jre_dir);
    }

    let os = get_adoptium_os();
    let arch = get_adoptium_arch();
    let url = format!("https://api.adoptium.net/v3/assets/latest/{}/hotspot?os={}&arch={}&image_type=jre", major_version, os, arch);

    let client = reqwest::blocking::Client::new();
    let response: Vec<AdoptiumAsset> = client.get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    if response.is_empty() {
        return Err(format!("No JRE found for version {}", major_version));
    }

    let download_url = &response[0].binary.package.link;
    let filename = format!("jre{}-{}-{}.{}", major_version, os, arch, if cfg!(target_os = "windows") { "zip" } else { "tar.gz" });
    let archive_path = jres_root.join(&filename);

    download_file_if_needed(download_url, &archive_path)?;

    // Extract the archive
    std::fs::create_dir_all(&jre_dir).map_err(|e| e.to_string())?;
    if cfg!(target_os = "windows") {
        // Use zip
        let file = std::fs::File::open(&archive_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let path = file.mangled_name();
            let out_path = jre_dir.join(path);
            if file.is_dir() {
                std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut out_file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut out_file).map_err(|e| e.to_string())?;
            }
        }
    } else {
        // Use tar.gz
        use std::process::Command;
        Command::new("tar")
            .args(&["-xzf", &archive_path.to_string_lossy(), "-C", &jre_dir.to_string_lossy()])
            .status()
            .map_err(|e| e.to_string())?;
    }

    // Clean up archive
    std::fs::remove_file(&archive_path).map_err(|e| e.to_string())?;

    Ok(jre_dir)
}

pub fn download_libraries(libraries_root: &Path, version_json: &str) -> Result<(), String> {
    let manifest: VersionManifest =
        serde_json::from_str(version_json).map_err(|e| e.to_string())?;

    println!("[Download] Total libraries in manifest: {}", manifest.libraries.len());

    for lib in manifest.libraries {
        match &lib.downloads.artifact {
            Some(artifact) => {
                let lib_name = &artifact.path;
                if is_library_allowed(&lib.rules) {
                    println!("[Download] Downloading allowed: {}", lib_name);
                    let _ = download_file_if_needed(&artifact.url, &libraries_root.join(&artifact.path));
                } else {
                    println!("[Download] Skipping (rules): {}", lib_name);
                }

                if let Some(natives_map) = &lib.natives {
                    let os_key = get_os_key();

                    if let Some(classifier_key) = natives_map.get(os_key) {
                        if let Some(classifiers) = &lib.downloads.classifiers {
                            if let Some(artifact) = classifiers.get(classifier_key) {
                                let _ = download_file_if_needed(
                                    &artifact.url,
                                    &libraries_root.join(&artifact.path),
                                );
                            }
                        }
                    }
                }
            }
            None => {
                println!("[Download] Library has no artifact entry");
            }
        }
    }
    Ok(())
}

pub fn extract_natives(
    libraries_root: &Path,
    natives_dir: &Path,
    version_manifest: &VersionManifest,
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
                                let mut archive =
                                    zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

                                for i in 0..archive.len() {
                                    let mut file =
                                        archive.by_index(i).map_err(|e| e.to_string())?;
                                    let path = file.mangled_name();

                                    if path.starts_with("META-INF") || file.is_dir() {
                                        continue;
                                    }

                                    let out_path = natives_dir.join(file.name());
                                    let mut out_file = std::fs::File::create(&out_path)
                                        .map_err(|e| e.to_string())?;
                                    std::io::copy(&mut file, &mut out_file)
                                        .map_err(|e| e.to_string())?;
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
