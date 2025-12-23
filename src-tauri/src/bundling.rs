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
    
    // Separate folders from files
    let mut folders_to_copy = HashSet::new();
    let mut files_to_copy = HashSet::new();
    
    for dep in sceneryobject_deps.iter()
        .chain(spline_deps.iter())
        .chain(human_deps.iter())
        .chain(vehicle_deps.iter())
        .chain(deps.textures.iter()) {
        
        if dep.starts_with("FOLDER:") {
            // This is a folder marker - extract the actual path
            let folder_path = dep.strip_prefix("FOLDER:").unwrap();
            folders_to_copy.insert(folder_path.to_string());
        } else {
            files_to_copy.insert(dep.clone());
        }
    }
    
    // Copy all files
    let mut copied_files = 0;
    let mut failed_files = Vec::new();
    
    for file_path in &files_to_copy {
        let src = omsi_root.join(file_path);
        let dest = temp_dir.join(file_path);
        
        if let Err(e) = copy_file_with_folders(&src, &dest) {
            failed_files.push(format!("{}: {}", file_path, e));
        } else {
            copied_files += 1;
        }
    }
    
    // Copy all folders
    let mut copied_folders = 0;
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
