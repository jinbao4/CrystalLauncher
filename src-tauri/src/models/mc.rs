use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionEntry {
    pub id: String,
    pub url: String,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub id: String, 

    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexInfo,
    #[serde(rename = "mainClass")]
    pub main_class: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetIndexInfo {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetMap {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetObject {
    pub hash: String,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Library {
    pub name: String,
    pub downloads: LibraryDownloads,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>, 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>, 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Downloads {
    pub client: ClientDownload,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientDownload {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OSRestriction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OSRestriction {
    pub name: String,
}