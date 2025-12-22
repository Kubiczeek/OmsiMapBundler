pub mod human;

use std::path::Path;
use std::collections::HashSet;

// Extract all nested dependencies for a given asset type
pub fn extract_nested_dependencies(
    asset_paths: &[String],
    asset_type: &str,
    omsi_root: &Path,
) -> HashSet<String> {
    let mut all_dependencies = HashSet::new();
    
    for asset_path in asset_paths {
        match asset_type {
            "human" => {
                if let Some(deps) = human::extract_human_dependencies(asset_path, omsi_root) {
                    all_dependencies.extend(deps);
                }
            }
            // TODO: Add more asset types (scenery objects, splines, vehicles, etc.)
            _ => {}
        }
    }
    
    all_dependencies
}
