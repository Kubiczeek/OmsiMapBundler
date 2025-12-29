use std::path::Path;
use std::collections::{HashSet, VecDeque};
use crate::phase2_extraction::{sco, sli, bus, ovh, hum, zug, cfg};

/// Process all dependencies starting from the initial set of files collected in Phase 1.
/// Returns a complete set of all files that need to be included in the bundle.
pub fn process_dependencies(initial_paths: HashSet<String>, omsi_root: &Path) -> HashSet<String> {
    let mut final_dependencies = HashSet::new();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    // Initialize queue with Phase 1 paths
    for path in initial_paths {
        if !visited.contains(&path) {
            visited.insert(path.clone());
            queue.push_back(path);
        }
    }

    while let Some(current_path) = queue.pop_front() {
        // Add to final result
        final_dependencies.insert(current_path.clone());

        // Check if it's a folder marker
        if current_path.starts_with("FOLDER:") {
            continue;
        }

        // Special handling for Money and TicketPacks folders
        // If we encounter a file in Money/ or TicketPacks/, we should include the parent folder
        if current_path.starts_with("Money\\") || current_path.starts_with("Money/") ||
           current_path.starts_with("TicketPacks\\") || current_path.starts_with("TicketPacks/") {
            if let Some(parent) = Path::new(&current_path).parent() {
                let folder_str = parent.to_string_lossy().replace('/', "\\");
                let folder_marker = format!("FOLDER:{}", folder_str);
                if !visited.contains(&folder_marker) {
                    visited.insert(folder_marker.clone());
                    queue.push_back(folder_marker);
                }
            }
        }

        // Determine file type and extract dependencies
        let path_lower = current_path.to_lowercase();
        let new_deps = if path_lower.ends_with(".sco") {
            sco::extract_sco_dependencies(&current_path, omsi_root)
        } else if path_lower.ends_with(".sli") {
            sli::extract_sli_dependencies(&current_path, omsi_root)
        } else if path_lower.ends_with(".bus") {
            bus::extract_bus_dependencies(&current_path, omsi_root)
        } else if path_lower.ends_with(".ovh") {
            // Check if it's in Vehicles or Sceneryobjects
            // But actually ovh logic is similar, usually AI vehicles
            ovh::extract_ovh_dependencies(&current_path, omsi_root)
        } else if path_lower.ends_with(".hum") {
            hum::extract_hum_dependencies(&current_path, omsi_root)
        } else if path_lower.ends_with(".zug") {
            zug::extract_zug_dependencies(&current_path, omsi_root)
        } else if path_lower.ends_with(".cfg") {
            // Generic config or sound config
            // We might need to distinguish, but for now let's try generic cfg extraction
            // which includes sound.cfg logic if we merge them or call both
            // For now, let's assume cfg.rs handles generic dependencies
            cfg::extract_cfg_dependencies(&current_path, omsi_root)
        } else {
            None
        };

        // Add new dependencies to queue
        if let Some(deps) = new_deps {
            for dep in deps {
                if !visited.contains(&dep) {
                    visited.insert(dep.clone());
                    queue.push_back(dep);
                }
            }
        }
    }

    final_dependencies
}
