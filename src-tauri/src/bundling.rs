use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use crate::types::{BundleRequest, BundleResult};
use crate::phase1_collection;
use crate::phase2_processing;
use crate::utils::{copy_file_with_folders, create_zip};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;
use walkdir::WalkDir;

pub type ProgressCallback = Box<dyn Fn(&str, f32) + Send + Sync>;

fn emit_progress(cb: &Option<Arc<ProgressCallback>>, message: &str, progress: f32) {
    if let Some(cb) = cb {
        cb(message, progress.clamp(0.0, 1.0));
    }
}

// Create the bundle ZIP file with all dependencies
pub fn create_bundle(request: BundleRequest, progress_cb: Option<Arc<ProgressCallback>>) -> BundleResult {
    let map_path = Path::new(&request.map_folder);
    
    // Find OMSI 2 root folder (should be 2 levels up from map folder: OMSI 2/maps/mapname)
    let omsi_root = match map_path.parent().and_then(|p| p.parent()) {
        Some(root) => root,
        None => return BundleResult {
            success: false,
            output_path: None,
            error: Some("Could not determine OMSI 2 root folder".to_string()),
        }
    };
    
    // Get map folder name
    let map_name = match map_path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return BundleResult {
            success: false,
            output_path: None,
            error: Some("Invalid map folder name".to_string()),
        }
    };
    
    println!("Bundling map: {}", map_name);
    
    // Phase 1: Collect initial dependencies from map files
    emit_progress(&progress_cb, "Scanning map configuration...", 0.01);
    let initial_deps = match phase1_collection::collect_all_dependencies(map_path) {
        Ok(deps) => deps,
        Err(e) => return BundleResult {
            success: false,
            output_path: None,
            error: Some(format!("Map scan failed: {}", e)),
        }
    };
    emit_progress(&progress_cb, format!("Map scan complete: {} files found", initial_deps.len()).as_str(), 0.1);
    
    // Create temp folder
    let temp_dir = std::env::temp_dir().join(format!("omsi_bundle_{}", map_name));
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return BundleResult {
            success: false,
            output_path: None,
            error: Some(format!("Failed to create temp folder: {}", e)),
        };
    }
    
    // Phase 2: Process dependencies recursively
    emit_progress(&progress_cb, "Analyzing dependencies...", 0.1);
    let all_deps = phase2_processing::process_dependencies(initial_deps, omsi_root);
    
    println!("Resolved {} total dependencies", all_deps.len());
    emit_progress(&progress_cb, format!("Analysis complete: {} total files", all_deps.len()).as_str(), 0.3);
    
    // Separate folders from files and flatten folders
    let mut files_to_copy = HashSet::new();
    let mut folders_to_process = Vec::new();
    
    for dep in &all_deps {
        if dep.starts_with("FOLDER:") {
            let folder_path = dep.strip_prefix("FOLDER:").unwrap();
            folders_to_process.push(folder_path.to_string());
        } else {
            files_to_copy.insert(dep.clone());
        }
    }

    // Flatten folders into files for better parallelism
    // We use par_iter to walk multiple folders at once if there are many
    let folder_files: Vec<String> = folders_to_process.par_iter().flat_map(|folder_rel_path| {
        let folder_abs_path = omsi_root.join(folder_rel_path);
        let mut files = Vec::new();
        if folder_abs_path.exists() {
             for entry in WalkDir::new(&folder_abs_path).into_iter().filter_map(|e| e.ok()) {
                 if entry.file_type().is_file() {
                     // Get path relative to omsi_root
                     if let Ok(rel) = entry.path().strip_prefix(omsi_root) {
                         files.push(rel.to_string_lossy().replace("\\", "/"));
                     }
                 }
             }
        }
        files
    }).collect();

    files_to_copy.extend(folder_files);

    // Filter out files that are inside the map folder itself (because we copy the whole map folder at the end)
    // map_path is absolute. We need to check if omsi_root/file is inside map_path
    // Or simpler: check if file path starts with "maps/mapname/"
    let map_folder_prefix = format!("maps/{}/", map_name);
    let final_files_list: Vec<String> = files_to_copy.into_iter()
        .filter(|f| !f.replace("\\", "/").to_lowercase().starts_with(&map_folder_prefix.to_lowercase()))
        .collect();
    
    // Copy all files with progress (Parallel)
    let copied_files = Arc::new(Mutex::new(0));
    let failed_files = Arc::new(Mutex::new(Vec::new()));
    let total_files = final_files_list.len().max(1);
    
    emit_progress(&progress_cb, "Copying assets...", 0.3);

    final_files_list.par_iter().for_each(|file_path| {
        let src = omsi_root.join(file_path);
        let dest = temp_dir.join(file_path);
        
        // Only attempt copy if source exists
        if src.exists() {
            if let Err(e) = copy_file_with_folders(&src, &dest) {
                failed_files.lock().unwrap().push(format!("{}: {}", file_path, e));
            } else {
                let mut count = copied_files.lock().unwrap();
                *count += 1;
                
                // Emit progress occasionally
                if *count % 100 == 0 {
                    let pct = 0.3 + 0.5 * (*count as f32 / total_files as f32);
                    emit_progress(&progress_cb, "Copying assets...", pct);
                }
            }
        }
    });
    
    // Final progress update for files
    emit_progress(&progress_cb, "Copying assets complete", 0.8);
    
    let final_copied_files = *copied_files.lock().unwrap();
    if total_files > 0 {
        println!("Copied {} files", final_copied_files);
    }
    
    // Copy entire map folder to temp/maps/mapname (Parallel)
    emit_progress(&progress_cb, "Copying map files...", 0.8);
    let map_dest = temp_dir.join("maps").join(map_name);
    
    // Collect all files in map folder
    let map_files: Vec<PathBuf> = WalkDir::new(map_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let total_map_files = map_files.len().max(1);
    let copied_map_files = Arc::new(Mutex::new(0));
    
    map_files.par_iter().for_each(|src_path| {
        // Calculate relative path
        if let Ok(rel_path) = src_path.strip_prefix(map_path) {
            let dest_path = map_dest.join(rel_path);
            
            if let Err(e) = copy_file_with_folders(src_path, &dest_path) {
                 failed_files.lock().unwrap().push(format!("Map file {}: {}", rel_path.display(), e));
            } else {
                let mut count = copied_map_files.lock().unwrap();
                *count += 1;
                
                if *count % 20 == 0 {
                    let pct = 0.8 + 0.15 * (*count as f32 / total_map_files as f32);
                    emit_progress(&progress_cb, "Copying map files...", pct);
                }
            }
        }
    });
    
    emit_progress(&progress_cb, "Map files copied", 0.95);
    
    // Copy README if specified
    if let Some(readme_path) = request.readme_path {
        let readme_src = Path::new(&readme_path);
        if readme_src.exists() {
            let readme_dest = temp_dir.join(readme_src.file_name().unwrap_or_default());
            let _ = fs::copy(readme_src, readme_dest);
        }
    }
    
    // Determine output path
    let zip_name = request.zip_name.unwrap_or_else(|| format!("{}.zip", map_name));
    
    // Ensure zip_name has .zip extension
    let zip_name = if !zip_name.to_lowercase().ends_with(".zip") {
        format!("{}.zip", zip_name)
    } else {
        zip_name
    };
    
    let output_path = if let Some(out_folder) = request.output_folder {
        PathBuf::from(out_folder).join(&zip_name)
    } else {
        map_path.join(&zip_name)
    };
    
    let compression_method = request
        .compression_method
        .as_deref()
        .unwrap_or("deflate")
        .to_lowercase();
    let compression_level = request.compression_level.unwrap_or(1);

    println!("Creating ZIP file...");
    emit_progress(&progress_cb, "Compressing bundle...", 0.95);
    
    // Create ZIP file
    match create_zip(&temp_dir, &output_path, &compression_method, compression_level) {
        Ok(_) => {
            // Clean up temp folder
            let _ = fs::remove_dir_all(&temp_dir);
            emit_progress(&progress_cb, "Done!", 1.0);
            
            println!("Bundle created: {}", output_path.display());
            
            BundleResult {
                success: true,
                output_path: Some(output_path.to_string_lossy().to_string()),
                error: {
                    let failed = failed_files.lock().unwrap();
                    if failed.is_empty() {
                        None
                    } else {
                        Some(format!("Bundle created, but {} files failed to copy", failed.len()))
                    }
                }
            }
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&temp_dir);
            BundleResult {
                success: false,
                output_path: None,
                error: Some(format!("Failed to create ZIP: {}", e)),
            }
        }
    }
}
