use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashSet;
use crate::types::{BundleRequest, BundleResult};
use crate::extraction::extract_dependencies;
use crate::utils::{copy_file_with_folders, copy_dir_all, create_zip};
use crate::dependencies;

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
    
    // Extract nested dependencies for humans
    println!("Extracting nested dependencies for humans...");
    let human_deps = dependencies::extract_nested_dependencies(&deps.humans, "human", omsi_root);
    println!("Found {} nested human dependencies", human_deps.len());
    
    // Combine all files to copy
    let mut all_files = HashSet::new();
    all_files.extend(deps.sceneryobjects.iter().cloned());
    all_files.extend(deps.splines.iter().cloned());
    all_files.extend(deps.textures.iter().cloned());
    all_files.extend(human_deps); // Use expanded human dependencies
    all_files.extend(deps.vehicles.iter().cloned());
    
    // Copy all dependencies
    let mut copied_files = 0;
    let mut failed_files = Vec::new();
    
    for file_path in &all_files {
        let src = omsi_root.join(file_path);
        let dest = temp_dir.join(file_path);
        
        // Debug: print full source path for texture files
        if file_path.starts_with("Texture\\") {
            println!("Trying to copy texture from: {:?}", src);
            println!("  Exists: {}", src.exists());
        }
        
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            println!("FAILED to copy {}: {}", file_path, e);
            failed_files.push(format!("{}: {}", file_path, e));
        } else {
            copied_files += 1;
        }
    }
    
    println!("Copied {} files, {} failed", copied_files, failed_files.len());
    if !failed_files.is_empty() {
        println!("Failed files:");
        for failed in &failed_files {
            println!("  - {}", failed);
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
