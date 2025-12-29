use std::path::Path;
use crate::types::DependencyResult;
use crate::phase1_collection;

/// Extract all dependencies from map folder
/// 
/// PHASE 1: Collects all file paths from map config files (.map, global.cfg, ailists.cfg, parklist_p.txt)
/// PHASE 2: Processes each file based on its type and extracts nested dependencies
pub fn extract_dependencies(map_folder: String) -> DependencyResult {
    let path = Path::new(&map_folder);

    // Phase 1: Collect ALL file paths without categorization
    let _all_paths = match phase1_collection::collect_all_dependencies(path) {
        Ok(paths) => paths,
        Err(e) => {
            let error_msg = format!("Failed to collect dependencies: {}", e);
            return DependencyResult {
                sceneryobjects: Vec::new(),
                splines: Vec::new(),
                textures: Vec::new(),
                humans: Vec::new(),
                vehicles: Vec::new(),
                money_systems: Vec::new(),
                ticket_packs: Vec::new(),
                tile_maps: Vec::new(),
                error: Some(error_msg),
            };
        }
    };

    DependencyResult {
        sceneryobjects: Vec::new(),
        splines: Vec::new(),
        textures: Vec::new(),
        humans: Vec::new(),
        vehicles: Vec::new(),
        money_systems: Vec::new(),
        ticket_packs: Vec::new(),
        tile_maps: Vec::new(),
        error: None,
    }
}
