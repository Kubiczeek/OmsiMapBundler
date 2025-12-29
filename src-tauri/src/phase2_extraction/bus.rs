use std::path::Path;
use std::collections::HashSet;

/// Extract all dependencies from a .bus file (vehicle configuration)
/// Returns a set with the vehicle folder path that needs to be copied entirely
pub fn extract_bus_dependencies(bus_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_bus_path = omsi_root.join(bus_path);
    
    if !full_bus_path.exists() {
        println!("Bus file not found: {:?}", full_bus_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Get the parent folder of the vehicle file
    let bus_file_path = Path::new(bus_path);
    if let Some(bus_folder) = bus_file_path.parent() {
        let folder_str = bus_folder.to_string_lossy().replace('/', "\\");
        
        // Safety check: don't copy empty or root paths
        if !folder_str.is_empty() && folder_str != "\\" && folder_str != "/" && folder_str.contains("\\") {
            // Add the entire vehicle folder
            // We'll use a special marker to indicate this is a folder, not a file
            dependencies.insert(format!("FOLDER:{}", folder_str));
            println!("  -> Will copy vehicle folder: {}", folder_str);
        } else {
            println!("  -> Skipping invalid vehicle folder path: '{}' from: '{}'", folder_str, bus_path);
        }
    }
    
    Some(dependencies)
}
