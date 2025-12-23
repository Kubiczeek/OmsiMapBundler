use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

/// Extract all dependencies from a .sli file
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_spline_dependencies(sli_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_sli_path = omsi_root.join(sli_path);
    
    if !full_sli_path.exists() {
        println!("Spline file not found: {:?}", full_sli_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .sli file itself
    dependencies.insert(sli_path.to_string());
    
    // Read .sli file with Windows-1252 encoding
    let sli_content = match File::open(&full_sli_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file);
            let mut content = String::new();
            match decoder.read_to_string(&mut content) {
                Ok(_) => content,
                Err(e) => {
                    println!("Failed to decode {}: {}", sli_path, e);
                    return None;
                }
            }
        }
        Err(e) => {
            println!("Failed to open {}: {}", sli_path, e);
            return None;
        }
    };
    
    let sli_folder = Path::new(sli_path).parent().unwrap_or(Path::new(""));
    let mut lines = sli_content.lines();
    
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Extract textures from [texture] sections
        if trimmed == "[texture]" {
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                if !tex_file.is_empty() && (tex_file.ends_with(".jpg") || 
                    tex_file.ends_with(".bmp") || tex_file.ends_with(".dds") || 
                    tex_file.ends_with(".png") || tex_file.ends_with(".tga")) {
                    
                    // Get base name without extension
                    let base_name = if let Some(pos) = tex_file.rfind('.') {
                        &tex_file[..pos]
                    } else {
                        tex_file
                    };
                    
                    // Find all texture variants (different extensions, night folder, etc.)
                    add_texture_variants(base_name, &sli_folder, omsi_root, &mut dependencies);
                }
            }
        }
    }
    
    Some(dependencies)
}

/// Helper function to find all texture variants with the same base name
fn add_texture_variants(base_name: &str, sli_folder: &Path, omsi_root: &Path, dependencies: &mut HashSet<String>) {
    use std::fs;
    
    let texture_extensions = ["jpg", "bmp", "dds", "png", "tga"];
    
    // Try multiple base locations
    let search_paths = vec![
        sli_folder.join("texture"),  // Splines\XYZ\texture\
        sli_folder.to_path_buf(),    // Splines\XYZ\
        Path::new("Texture").to_path_buf(), // Global Texture\
    ];
    
    for search_path in search_paths {
        let full_search_path = omsi_root.join(&search_path);
        
        if !full_search_path.exists() || !full_search_path.is_dir() {
            continue;
        }
        
        // Search in main folder
        for ext in &texture_extensions {
            let file_name = format!("{}.{}", base_name, ext);
            let file_path = search_path.join(&file_name);
            let full_file_path = omsi_root.join(&file_path);
            
            if full_file_path.exists() {
                let path_str = file_path.to_string_lossy().replace('/', "\\");
                dependencies.insert(path_str.clone());
                
                // Also add .cfg and .surf files if they exist
                let cfg_path = format!("{}.cfg", path_str);
                let surf_path = format!("{}.surf", path_str);
                
                if omsi_root.join(&cfg_path).exists() {
                    dependencies.insert(cfg_path);
                }
                if omsi_root.join(&surf_path).exists() {
                    dependencies.insert(surf_path);
                }
            }
            
            // Also check case-insensitive match
            let file_name_lower = format!("{}.{}", base_name.to_lowercase(), ext);
            if file_name.to_lowercase() != file_name_lower {
                let file_path_ci = search_path.join(&file_name_lower);
                let full_file_path_ci = omsi_root.join(&file_path_ci);
                
                if full_file_path_ci.exists() {
                    let path_str = file_path_ci.to_string_lossy().replace('/', "\\");
                    dependencies.insert(path_str.clone());
                    
                    // Also add .cfg and .surf files if they exist
                    let cfg_path = format!("{}.cfg", path_str);
                    let surf_path = format!("{}.surf", path_str);
                    
                    if omsi_root.join(&cfg_path).exists() {
                        dependencies.insert(cfg_path);
                    }
                    if omsi_root.join(&surf_path).exists() {
                        dependencies.insert(surf_path);
                    }
                }
            }
        }
        
        // Search in subfolders (like night\, alpha\, etc.)
        if let Ok(entries) = fs::read_dir(&full_search_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let subfolder_name = entry_path.file_name().unwrap().to_string_lossy().to_string();
                    
                    for ext in &texture_extensions {
                        let file_name = format!("{}.{}", base_name, ext);
                        let file_path = search_path.join(&subfolder_name).join(&file_name);
                        let full_file_path = omsi_root.join(&file_path);
                        
                        if full_file_path.exists() {
                            let path_str = file_path.to_string_lossy().replace('/', "\\");
                            dependencies.insert(path_str.clone());
                            
                            // Also add .cfg and .surf files if they exist
                            let cfg_path = format!("{}.cfg", path_str);
                            let surf_path = format!("{}.surf", path_str);
                            
                            if omsi_root.join(&cfg_path).exists() {
                                dependencies.insert(cfg_path);
                            }
                            if omsi_root.join(&surf_path).exists() {
                                dependencies.insert(surf_path);
                            }
                        }
                        
                        // Case-insensitive check
                        let file_name_lower = format!("{}.{}", base_name.to_lowercase(), ext);
                        if file_name.to_lowercase() != file_name_lower {
                            let file_path_ci = search_path.join(&subfolder_name).join(&file_name_lower);
                            let full_file_path_ci = omsi_root.join(&file_path_ci);
                            
                            if full_file_path_ci.exists() {
                                let path_str = file_path_ci.to_string_lossy().replace('/', "\\");
                                dependencies.insert(path_str.clone());
                                
                                // Also add .cfg and .surf files if they exist
                                let cfg_path = format!("{}.cfg", path_str);
                                let surf_path = format!("{}.surf", path_str);
                                
                                if omsi_root.join(&cfg_path).exists() {
                                    dependencies.insert(cfg_path);
                                }
                                if omsi_root.join(&surf_path).exists() {
                                    dependencies.insert(surf_path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
