pub mod human;
pub mod sceneryobject;
pub mod spline;
pub mod train;
pub mod vehicle;

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
            "sceneryobject" => {
                // .ovh files in Sceneryobjects are AI vehicles and need special handling
                if asset_path.ends_with(".ovh") {
                    if let Some(deps) = sceneryobject::extract_ovh_dependencies(asset_path, omsi_root) {
                        all_dependencies.extend(deps);
                    }
                } else if let Some(deps) = sceneryobject::extract_sceneryobject_dependencies(asset_path, omsi_root) {
                    all_dependencies.extend(deps);
                }
            }
            "spline" => {
                if let Some(deps) = spline::extract_spline_dependencies(asset_path, omsi_root) {
                    all_dependencies.extend(deps);
                }
            }
            "vehicle" => {
                // Distinguish between trains (.zug) and buses/vehicles (.bus/.ovh/.sco)
                if asset_path.ends_with(".zug") {
                    if let Some(deps) = train::extract_train_dependencies(asset_path, omsi_root) {
                        all_dependencies.extend(deps);
                    }
                } else if asset_path.ends_with(".bus") || asset_path.ends_with(".ovh") || asset_path.ends_with(".sco") {
                    if let Some(deps) = vehicle::extract_vehicle_dependencies(asset_path, omsi_root) {
                        all_dependencies.extend(deps);
                    }
                }
            }
            // TODO: Add more asset types
            _ => {}
        }
    }
    
    all_dependencies
}
