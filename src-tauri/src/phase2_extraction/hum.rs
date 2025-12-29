use std::path::Path;
use std::collections::HashSet;
use crate::phase2_extraction::{utils, cfg};

/// Extract all dependencies from a .hum file
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_hum_dependencies(hum_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_hum_path = omsi_root.join(hum_path);
    
    if !full_hum_path.exists() {
        println!("Human file not found: {:?}", full_hum_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .hum file itself
    dependencies.insert(hum_path.to_string());
    
    // Read .hum file with Windows-1252 encoding
    let hum_content = match utils::read_file_windows1252(&full_hum_path) {
        Some(content) => content,
        None => return None,
    };
    
    // Parse [model] section to find the .cfg file
    let mut lines = hum_content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        if trimmed == "[model]" {
            // Next line contains the model .cfg path
            if let Some(cfg_line) = lines.next() {
                let cfg_path = cfg_line.trim();
                
                if !cfg_path.is_empty() {
                    // Build full path relative to human folder
                    let human_folder = Path::new(hum_path).parent().unwrap_or(Path::new(""));
                    let full_cfg_path = human_folder.join(cfg_path);
                    let cfg_path_str = full_cfg_path.to_string_lossy().replace('/', "\\");
                    
                    dependencies.insert(cfg_path_str.clone());
                    
                    // Extract dependencies from the .cfg file
                    if let Some(cfg_deps) = cfg::extract_cfg_dependencies(&cfg_path_str, omsi_root) {
                        dependencies.extend(cfg_deps);
                    }
                }
            }
        }
    }
    
    Some(dependencies)
}
