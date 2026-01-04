use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub missing_files: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyResult {
    pub sceneryobjects: Vec<String>,
    pub splines: Vec<String>,
    pub textures: Vec<String>,
    pub humans: Vec<String>,
    pub vehicles: Vec<String>,
    pub money_systems: Vec<String>,
    pub ticket_packs: Vec<String>,
    pub tile_maps: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleRequest {
    pub map_folder: String,
    pub addon_folder: Option<String>,
    pub output_folder: Option<String>,
    pub zip_name: Option<String>,
    pub readme_path: Option<String>,
    pub compression_method: Option<String>,
    pub compression_level: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}
