use std::path::{Path, PathBuf};
use std::fs;
use crate::types::{BundleRequest, BundleResult};
use crate::extraction::extract_dependencies;
use crate::utils::{copy_file_with_folders, copy_dir_all, create_zip};

// Create the bundle ZIP file with all dependencies
pub fn create_bundle(request: BundleRequest) -> BundleResult {
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
    let deps = extract_dependencies(request.map_folder.clone());
    if let Some(err) = deps.error {
        return BundleResult {
            success: false,
            output_path: None,
            error: Some(format!("Failed to extract dependencies: {}", err)),
        };
    }
    
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
    
    // Copy all dependencies
    let mut copied_files = 0;
    let mut failed_files = Vec::new();
    
    // Copy scenery objects
    for obj in &deps.sceneryobjects {
        let src = omsi_root.join(obj);
        let dest = temp_dir.join(obj);
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", obj, e));
        } else {
            copied_files += 1;
        }
    }
    
    // Copy splines
    for spline in &deps.splines {
        let src = omsi_root.join(spline);
        let dest = temp_dir.join(spline);
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", spline, e));
        } else {
            copied_files += 1;
        }
    }
    
    // Copy textures
    for texture in &deps.textures {
        let src = omsi_root.join(texture);
        let dest = temp_dir.join(texture);
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", texture, e));
        } else {
            copied_files += 1;
        }
    }
    
    // Copy humans
    for human in &deps.humans {
        let src = omsi_root.join(human);
        let dest = temp_dir.join(human);
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", human, e));
        } else {
            copied_files += 1;
        }
    }
    
    // Copy vehicles
    for vehicle in &deps.vehicles {
        let src = omsi_root.join(vehicle);
        let dest = temp_dir.join(vehicle);
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", vehicle, e));
        } else {
            copied_files += 1;
        }
    }
    
    println!("Copied {} files, {} failed", copied_files, failed_files.len());
    
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
    let output_path = if let Some(out_folder) = request.output_folder {
        PathBuf::from(out_folder).join(&zip_name)
    } else {
        map_path.join(&zip_name)
    };
    
    println!("Creating ZIP: {:?}", output_path);
    
    // Create ZIP file
    match create_zip(&temp_dir, &output_path) {
        Ok(_) => {
            // Clean up temp folder
            let _ = fs::remove_dir_all(&temp_dir);
            
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
