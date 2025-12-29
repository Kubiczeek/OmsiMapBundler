use std::path::Path;
use std::collections::HashSet;
use crate::phase2_extraction::utils;

/// Extract all dependencies from a .zug file (train configuration)
/// Returns a set of vehicle folder paths that need to be copied entirely
pub fn extract_zug_dependencies(zug_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_zug_path = omsi_root.join(zug_path);
    
    if !full_zug_path.exists() {
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .zug file itself
    dependencies.insert(zug_path.to_string());
    
    // Read .zug file with Windows-1252 encoding
    let zug_content = match utils::read_file_windows1252(&full_zug_path) {
        Some(content) => content,
        None => return None,
    };
    
    // Parse .zug file - every other line starting from first is a vehicle path
    let lines: Vec<&str> = zug_content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Check if line looks like a vehicle path (contains .ovh or .bus)
        if line.ends_with(".ovh") || line.ends_with(".bus") {
            // Get the parent folder of the vehicle file
            let vehicle_path = Path::new(line);
            if let Some(vehicle_folder) = vehicle_path.parent() {
                let folder_str = vehicle_folder.to_string_lossy().replace('/', "\\");
                
                // Safety check: don't copy empty or root paths
                if !folder_str.is_empty() && folder_str != "\\" && folder_str != "/" && folder_str.contains("\\") {
                    // Add the entire vehicle folder
                    // We'll use a special marker to indicate this is a folder, not a file
                    dependencies.insert(format!("FOLDER:{}", folder_str));
                }
            }
            
            // Skip the next line (configuration number)
            i += 2;
        } else {
            i += 1;
        }
    }
    
    Some(dependencies)
}
