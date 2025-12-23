use std::path::Path;
use std::collections::HashSet;

/// Extract all dependencies from a .bus or .ovh file (vehicle configuration)
/// Returns a set with the vehicle folder path that needs to be copied entirely
pub fn extract_vehicle_dependencies(vehicle_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_vehicle_path = omsi_root.join(vehicle_path);
    
    if !full_vehicle_path.exists() {
        println!("Vehicle file not found: {:?}", full_vehicle_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Get the parent folder of the vehicle file
    let vehicle_file_path = Path::new(vehicle_path);
    if let Some(vehicle_folder) = vehicle_file_path.parent() {
        let folder_str = vehicle_folder.to_string_lossy().replace('/', "\\");
        
        // Safety check: don't copy empty or root paths
        if !folder_str.is_empty() && folder_str != "\\" && folder_str != "/" && folder_str.contains("\\") {
            // Add the entire vehicle folder
            // We'll use a special marker to indicate this is a folder, not a file
            dependencies.insert(format!("FOLDER:{}", folder_str));
            println!("  -> Will copy vehicle folder: {}", folder_str);
        } else {
            println!("  -> Skipping invalid vehicle folder path: '{}' from: '{}'", folder_str, vehicle_path);
        }
    }
    
    Some(dependencies)
}
