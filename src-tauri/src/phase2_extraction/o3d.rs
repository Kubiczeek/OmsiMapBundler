use std::path::Path;
use std::fs;
use std::io::Read;
use crate::phase2_extraction::utils;

/// Extract texture references from .o3d binary file
pub fn extract_o3d_textures(o3d_path: &str, omsi_root: &Path) -> Option<Vec<String>> {
    let full_o3d_path = omsi_root.join(o3d_path);
    
    if !full_o3d_path.exists() {
        return None;
    }
    
    // Read binary file
    let mut file = match fs::File::open(&full_o3d_path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    
    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        return None;
    }
    
    utils::extract_textures_from_binary(&buffer)
}
