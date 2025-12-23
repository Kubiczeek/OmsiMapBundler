use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

/// Extract all dependencies from a .zug file (train configuration)
/// Returns a set of vehicle folder paths that need to be copied entirely
pub fn extract_train_dependencies(zug_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_zug_path = omsi_root.join(zug_path);
    
    if !full_zug_path.exists() {
        println!("Train file not found: {:?}", full_zug_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .zug file itself
    dependencies.insert(zug_path.to_string());
    
    // Read .zug file with Windows-1252 encoding
    let zug_content = match File::open(&full_zug_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file);
            let mut content = String::new();
            match decoder.read_to_string(&mut content) {
                Ok(_) => content,
                Err(e) => {
                    println!("Failed to decode {}: {}", zug_path, e);
                    return None;
                }
            }
        }
        Err(e) => {
            println!("Failed to open {}: {}", zug_path, e);
            return None;
        }
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
                    println!("  -> Will copy vehicle folder: {}", folder_str);
                } else {
                    println!("  -> Skipping invalid vehicle folder path: '{}' from line: '{}'", folder_str, line);
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
