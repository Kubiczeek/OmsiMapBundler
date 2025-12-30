use std::path::Path;
use std::collections::HashSet;
use rayon::prelude::*;
use crate::phase2_extraction::{sco, sli, bus, ovh, hum, zug, cfg};

/// Process all dependencies starting from the initial set of files collected in Phase 1.
/// Returns a complete set of all files that need to be included in the bundle.
pub fn process_dependencies(initial_paths: HashSet<String>, omsi_root: &Path) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue: Vec<String> = Vec::new();

    // Initialize queue with Phase 1 paths
    for path in initial_paths {
        if !visited.contains(&path) {
            visited.insert(path.clone());
            queue.push(path);
        }
    }

    while !queue.is_empty() {
        // Take current batch from queue
        let current_batch = std::mem::take(&mut queue);

        // Process batch in parallel
        let results: Vec<HashSet<String>> = current_batch.par_iter()
            .map(|current_path| {
                let mut new_items = HashSet::new();

                // Check if it's a folder marker
                if current_path.starts_with("FOLDER:") {
                    return new_items;
                }

                // Special handling for Money and TicketPacks folders
                if current_path.starts_with("Money\\") || current_path.starts_with("Money/") ||
                   current_path.starts_with("TicketPacks\\") || current_path.starts_with("TicketPacks/") {
                    if let Some(parent) = Path::new(&current_path).parent() {
                        let folder_str = parent.to_string_lossy().replace('/', "\\");
                        let folder_marker = format!("FOLDER:{}", folder_str);
                        new_items.insert(folder_marker);
                    }
                }

                // Determine file type and extract dependencies
                let path_lower = current_path.to_lowercase();
                let extracted = if path_lower.ends_with(".sco") {
                    sco::extract_sco_dependencies(&current_path, omsi_root)
                } else if path_lower.ends_with(".sli") {
                    sli::extract_sli_dependencies(&current_path, omsi_root)
                } else if path_lower.ends_with(".bus") {
                    bus::extract_bus_dependencies(&current_path, omsi_root)
                } else if path_lower.ends_with(".ovh") {
                    ovh::extract_ovh_dependencies(&current_path, omsi_root)
                } else if path_lower.ends_with(".hum") {
                    hum::extract_hum_dependencies(&current_path, omsi_root)
                } else if path_lower.ends_with(".zug") {
                    zug::extract_zug_dependencies(&current_path, omsi_root)
                } else if path_lower.ends_with(".cfg") {
                    cfg::extract_cfg_dependencies(&current_path, omsi_root)
                } else {
                    None
                };

                if let Some(deps) = extracted {
                    for dep in deps {
                        new_items.insert(dep);
                    }
                }
                
                new_items
            })
            .collect();

        // Merge results back into queue (single threaded part)
        for result_set in results {
            for item in result_set {
                if !visited.contains(&item) {
                    visited.insert(item.clone());
                    queue.push(item);
                }
            }
        }
    }

    visited
}
