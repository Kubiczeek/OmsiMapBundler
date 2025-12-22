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
    pub tile_maps: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleRequest {
    pub map_folder: String,
    pub output_folder: Option<String>,
    pub zip_name: Option<String>,
    pub readme_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}
