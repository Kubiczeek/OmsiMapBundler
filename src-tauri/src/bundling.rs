use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use crate::types::{BundleRequest, BundleResult};
use crate::phase1_collection;
use crate::phase2_processing;
use crate::utils::{copy_file_with_folders, copy_dir_all, create_zip};
use std::sync::Arc;

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
    emit_progress(&progress_cb, "Phase 1: Collecting map files", 0.05);
    let initial_deps = match phase1_collection::collect_all_dependencies(map_path) {
        Ok(deps) => deps,
        Err(e) => return BundleResult {
            success: false,
            output_path: None,
            error: Some(format!("Phase 1 failed: {}", e)),
        }
    };
    emit_progress(&progress_cb, format!("Phase 1 complete: {} files found", initial_deps.len()).as_str(), 0.1);
    
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
    emit_progress(&progress_cb, "Phase 2: Processing dependencies", 0.15);
    let all_deps = phase2_processing::process_dependencies(initial_deps, omsi_root);
    
    println!("Resolved {} total dependencies", all_deps.len());
    emit_progress(&progress_cb, format!("Phase 2 complete: {} total files", all_deps.len()).as_str(), 0.25);
    
    // Separate folders from files
    let mut folders_to_copy = HashSet::new();
    let mut files_to_copy = HashSet::new();
    
    for dep in &all_deps {
        if dep.starts_with("FOLDER:") {
            // This is a folder marker - extract the actual path
            let folder_path = dep.strip_prefix("FOLDER:").unwrap();
            folders_to_copy.insert(folder_path.to_string());
        } else {
            files_to_copy.insert(dep.clone());
        }
    }
    
    // Copy all files with progress
    let mut copied_files = 0;
    let mut failed_files = Vec::new();
    let total_files = files_to_copy.len().max(1);
    for file_path in &files_to_copy {
        let src = omsi_root.join(file_path);
        let dest = temp_dir.join(file_path);
        
        // Only attempt copy if source exists
        if src.exists() {
            if let Err(e) = copy_file_with_folders(&src, &dest) {
                failed_files.push(format!("{}: {}", file_path, e));
            } else {
                copied_files += 1;
            }
        } else {
            // println!("Missing file: {}", file_path);
        }

        if copied_files % 50 == 0 || copied_files == files_to_copy.len() {
            let pct = 0.25 + 0.55 * (copied_files as f32 / total_files as f32);
            emit_progress(&progress_cb, "Copying files", pct);
        }
    }
    
    if total_files > 0 {
        println!("Copied {} files", copied_files);
    }
    
    // Copy all folders with progress
    let mut copied_folders = 0;
    let total_folders = folders_to_copy.len().max(1);
    for folder_path in &folders_to_copy {
        let src = omsi_root.join(folder_path);
        let dest = temp_dir.join(folder_path);
        
        if src.exists() {
            if let Ok(_) = copy_dir_all(&src, &dest) {
                copied_folders += 1;
            }
        }

        if copied_folders % 5 == 0 || copied_folders == folders_to_copy.len() {
            let pct = 0.8 + 0.1 * (copied_folders as f32 / total_folders as f32);
            emit_progress(&progress_cb, "Copying folders", pct);
        }
    }
    
    if total_folders > 0 {
        println!("Copied {} folders", copied_folders);
    }
    
    // Copy entire map folder to temp/maps/mapname
    let map_dest = temp_dir.join("maps").join(map_name);
    if let Err(e) = copy_dir_all(map_path, &map_dest) {
        let _ = fs::remove_dir_all(&temp_dir);
        return BundleResult {
            success: false,
            output_path: None,
            error: Some(format!("Failed to copy map folder: {}", e)),
        };
    }
    
    emit_progress(&progress_cb, "Copied map folder", 0.92);
    
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
    emit_progress(&progress_cb, "Creating ZIP", 0.94);
    
    // Create ZIP file
    match create_zip(&temp_dir, &output_path, &compression_method, compression_level) {
        Ok(_) => {
            // Clean up temp folder
            let _ = fs::remove_dir_all(&temp_dir);
            emit_progress(&progress_cb, "Finished", 1.0);
            
            println!("Bundle created: {}", output_path.display());
            
            BundleResult {
                success: true,
                output_path: Some(output_path.to_string_lossy().to_string()),
                error: if failed_files.is_empty() {
                    None
                } else {
                    Some(format!("Bundle created, but {} files failed to copy", failed_files.len()))
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
