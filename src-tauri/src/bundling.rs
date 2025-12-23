use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use crate::types::{BundleRequest, BundleResult};
use crate::extraction::extract_dependencies;
use crate::utils::{copy_file_with_folders, copy_dir_all, create_zip};
use std::sync::Arc;

pub type ProgressCallback = Box<dyn Fn(&str, f32) + Send + Sync>;

fn emit_progress(cb: &Option<Arc<ProgressCallback>>, message: &str, progress: f32) {
    if let Some(cb) = cb {
        cb(message, progress.clamp(0.0, 1.0));
    }
}
use crate::dependencies;

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
    
    println!("OMSI 2 root folder: {:?}", omsi_root);
    
    // Get map folder name
    let map_name = match map_path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return BundleResult {
            success: false,
            output_path: None,
            error: Some("Invalid map folder name".to_string()),
        }
    };
    
    // Extract dependencies first
    emit_progress(&progress_cb, "Extracting dependencies", 0.05);
    let deps = extract_dependencies(request.map_folder.clone());
    if let Some(err) = deps.error {
        return BundleResult {
            success: false,
            output_path: None,
            error: Some(format!("Failed to extract dependencies: {}", err)),
        };
    }
    emit_progress(&progress_cb, "Dependencies extracted", 0.1);
    
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
    
    println!("Created temp folder: {:?}", temp_dir);
    
    // Extract nested dependencies for humans
    println!("Extracting nested dependencies for humans...");
    let human_deps = dependencies::extract_nested_dependencies(&deps.humans, "human", omsi_root);
    println!("Found {} nested human dependencies", human_deps.len());
    
    // Extract nested dependencies for sceneryobjects
    println!("Extracting nested dependencies for sceneryobjects...");
    let sceneryobject_deps = dependencies::extract_nested_dependencies(&deps.sceneryobjects, "sceneryobject", omsi_root);
    println!("Found {} nested sceneryobject dependencies", sceneryobject_deps.len());
    
    // Extract nested dependencies for splines
    println!("Extracting nested dependencies for splines...");
    let spline_deps = dependencies::extract_nested_dependencies(&deps.splines, "spline", omsi_root);
    println!("Found {} nested spline dependencies", spline_deps.len());
    
    // Extract nested dependencies for vehicles (trains)
    println!("Extracting nested dependencies for vehicles...");
    let vehicle_deps = dependencies::extract_nested_dependencies(&deps.vehicles, "vehicle", omsi_root);
    println!("Found {} nested vehicle dependencies", vehicle_deps.len());
    emit_progress(&progress_cb, "Resolving nested dependencies", 0.25);
    
    // Separate folders from files
    let mut folders_to_copy = HashSet::new();
    let mut files_to_copy = HashSet::new();
    
    for dep in sceneryobject_deps.iter()
        .chain(spline_deps.iter())
        .chain(human_deps.iter())
        .chain(vehicle_deps.iter())
        .chain(deps.textures.iter())
        .chain(deps.money_systems.iter())
        .chain(deps.ticket_packs.iter()) {
        
        if dep.starts_with("FOLDER:") {
            // This is a folder marker - extract the actual path
            let folder_path = dep.strip_prefix("FOLDER:").unwrap();
            folders_to_copy.insert(folder_path.to_string());
        } else {
            files_to_copy.insert(dep.clone());
        }
    }

    // Ensure money systems and ticket packs copy their whole folder
    for money_path in &deps.money_systems {
        if let Some(parent) = Path::new(money_path).parent() {
            folders_to_copy.insert(parent.to_string_lossy().to_string());
        }
        files_to_copy.insert(money_path.clone());
    }

    for ticket_path in &deps.ticket_packs {
        if let Some(parent) = Path::new(ticket_path).parent() {
            folders_to_copy.insert(parent.to_string_lossy().to_string());
        }
        files_to_copy.insert(ticket_path.clone());
    }
    
    // Copy all files with progress
    let mut copied_files = 0;
    let mut failed_files = Vec::new();
    let total_files = files_to_copy.len().max(1);
    for file_path in &files_to_copy {
        let src = omsi_root.join(file_path);
        let dest = temp_dir.join(file_path);
        
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", file_path, e));
        } else {
            copied_files += 1;
        }

        if copied_files % 25 == 0 || copied_files == files_to_copy.len() {
            let pct = 0.25 + 0.55 * (copied_files as f32 / total_files as f32);
            emit_progress(&progress_cb, "Copying files", pct);
        }
    }
    
    // Copy all folders with progress
    let mut copied_folders = 0;
    let total_folders = folders_to_copy.len().max(1);
    println!("Preparing to copy {} vehicle folders...", folders_to_copy.len());
    for folder_path in &folders_to_copy {
        println!("  Copying vehicle folder: {}", folder_path);
        let src = omsi_root.join(folder_path);
        let dest = temp_dir.join(folder_path);
        
        if src.exists() {
            if let Ok(_) = copy_dir_all(&src, &dest) {
                copied_folders += 1;
                println!("Copied vehicle folder: {}", folder_path);
            } else {
                println!("Warning: Failed to copy vehicle folder: {}", folder_path);
            }
        }

        if copied_folders % 5 == 0 || copied_folders == folders_to_copy.len() {
            let pct = 0.8 + 0.1 * (copied_folders as f32 / total_folders as f32);
            emit_progress(&progress_cb, "Copying folders", pct);
        }
    }
    
    println!("Copied {} files and {} folders successfully", copied_files, copied_folders);
    if !failed_files.is_empty() {
        println!("Warning: {} optional files were not found (this is usually normal):", failed_files.len());
        for (idx, failed) in failed_files.iter().enumerate() {
            if idx < 5 {
                println!("  - {}", failed);
            }
        }
        if failed_files.len() > 5 {
            println!("  ... and {} more", failed_files.len() - 5);
        }
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
    
    println!("Copied map folder to temp");
    emit_progress(&progress_cb, "Copied map folder", 0.92);
    
    // Copy README if specified
    if let Some(readme_path) = request.readme_path {
        let readme_src = Path::new(&readme_path);
        if readme_src.exists() {
            let readme_dest = temp_dir.join(readme_src.file_name().unwrap_or_default());
            let _ = fs::copy(readme_src, readme_dest);
            println!("Copied README");
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

    println!(
        "Creating ZIP: {:?} (method: {}, level: {})",
        output_path, compression_method, compression_level
    );
    emit_progress(&progress_cb, "Creating ZIP", 0.94);
    
    // Create ZIP file
    match create_zip(&temp_dir, &output_path, &compression_method, compression_level) {
        Ok(_) => {
            // Clean up temp folder
            let _ = fs::remove_dir_all(&temp_dir);
            emit_progress(&progress_cb, "Finished", 1.0);
            
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
