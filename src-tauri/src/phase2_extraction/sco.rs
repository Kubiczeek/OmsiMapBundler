use std::path::Path;
use std::collections::HashSet;
use crate::phase2_extraction::{utils, o3d, x};

/// Extract all dependencies from a .sco file
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_sco_dependencies(sco_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_sco_path = omsi_root.join(sco_path);
    
    if !full_sco_path.exists() {
        println!("Sceneryobject file not found: {:?}", full_sco_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .sco file itself
    dependencies.insert(sco_path.to_string());
    
    // Read .sco file with Windows-1252 encoding
    let sco_content = match utils::read_file_windows1252(&full_sco_path) {
        Some(content) => content,
        None => return None,
    };
    
    let sco_folder = Path::new(sco_path).parent().unwrap_or(Path::new(""));
    
    let mut lines = sco_content.lines();
    
    while let Some(line) = lines.next() {
        // Clean the line: remove null bytes (common in bad encodings) and trim
        let cleaned_line = line.replace('\0', "");
        let trimmed = cleaned_line.trim();
        
        let section = trimmed.to_lowercase();
        
        // Extract model configuration from [model] sections
        if section == "[model]" {
            if let Some(_model_line) = lines.next() {
                // Found [model] tag - treat as complex object and copy entire folder
                let folder_str = sco_folder.to_string_lossy().replace('/', "\\");
                if !folder_str.is_empty() {
                    dependencies.insert(format!("FOLDER:{}", folder_str));
                }
            }
        }
        
        // Extract mesh files from [mesh] sections
        if section == "[mesh]" {
            if let Some(mesh_line) = lines.next() {
                let mesh_file_raw = mesh_line.trim();
                // Handle potential inline comments or whitespace issues
                let mesh_file = mesh_file_raw.split(&[';', '#'][..]).next().unwrap_or("").trim();
                
                let mesh_file_lower = mesh_file.to_lowercase();
                if !mesh_file.is_empty() && (mesh_file_lower.ends_with(".o3d") || mesh_file_lower.ends_with(".x")) {
                    // Determine the absolute path to the SCO folder
                    let abs_sco_folder = omsi_root.join(sco_folder);
                    
                    // Define search candidates
                    let candidates = vec![
                        abs_sco_folder.join("model"),
                        abs_sco_folder.join("Model"),
                        abs_sco_folder.clone(),
                    ];

                    // Try to find the file in the candidates
                    let mut found_path = None;
                    for candidate in &candidates {
                        if let Some(p) = utils::find_file(candidate, mesh_file) {
                            found_path = Some(p);
                            break;
                        }
                    }

                    let mesh_path = if let Some(abs_path) = found_path {
                        // Convert absolute path back to relative path for dependencies set
                        if let Some(rel) = utils::make_relative_path(&abs_path, omsi_root) {
                            rel
                        } else {
                            eprintln!("WARNING: Could not strip prefix {:?} from {:?}. Fallback used.", omsi_root, abs_path);
                            sco_folder.join(mesh_file).to_string_lossy().replace('/', "\\")
                        }
                    } else {
                        // Log missing
                        log_missing_file(sco_path, "Mesh", mesh_file, &candidates);
                        // Fallback: use as-is relative to sco folder
                        sco_folder.join(mesh_file).to_string_lossy().replace('/', "\\")
                    };
                    
                    dependencies.insert(mesh_path.clone());
                    
                    // Extract textures embedded in mesh files
                    let mesh_textures = if mesh_file_lower.ends_with(".o3d") {
                        o3d::extract_o3d_textures(&mesh_path, omsi_root)
                    } else if mesh_file_lower.ends_with(".x") {
                        x::extract_x_textures(&mesh_path, omsi_root)
                    } else {
                        None
                    };
                    
                    let mut textures_found = false;
                    if let Some(textures) = &mesh_textures {
                        if !textures.is_empty() {
                            textures_found = true;
                            for tex_name in textures {
                                // Get base name without extension
                                let base_name = if let Some(pos) = tex_name.rfind('.') {
                                    &tex_name[..pos]
                                } else {
                                    &tex_name
                                };
                                
                                // Find all texture variants
                                utils::add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                            }
                        }
                    }
                    
                    if !textures_found {
                        // Fallback for .o3d files: if no textures found, copy the object's Texture folder
                        if mesh_file_lower.ends_with(".o3d") {
                            // Add Texture folder (standard)
                            let texture_folder = sco_folder.join("Texture");
                            let texture_folder_str = texture_folder.to_string_lossy().replace('/', "\\");
                            dependencies.insert(format!("FOLDER:{}", texture_folder_str));
                            
                            // Add texture folder (lowercase)
                            let texture_folder_lower = sco_folder.join("texture");
                            let texture_folder_lower_str = texture_folder_lower.to_string_lossy().replace('/', "\\");
                            dependencies.insert(format!("FOLDER:{}", texture_folder_lower_str));
                        }
                    }
                }
            }
        }
        
        // Extract collision mesh files from [collision_mesh] sections
        if section == "[collision_mesh]" {
            if let Some(mesh_line) = lines.next() {
                let mesh_file_raw = mesh_line.trim();
                let mesh_file = mesh_file_raw.split(&[';', '#'][..]).next().unwrap_or("").trim();
                
                let mesh_file_lower = mesh_file.to_lowercase();
                if !mesh_file.is_empty() && (mesh_file_lower.ends_with(".o3d") || mesh_file_lower.ends_with(".x")) {
                    // Determine the absolute path to the SCO folder
                    let abs_sco_folder = omsi_root.join(sco_folder);
                    
                    let candidates = vec![
                        abs_sco_folder.join("model"),
                        abs_sco_folder.join("Model"),
                        abs_sco_folder.clone(),
                    ];

                    let mut found_path = None;
                    for candidate in &candidates {
                        if let Some(p) = utils::find_file(candidate, mesh_file) {
                            found_path = Some(p);
                            break;
                        }
                    }

                    let mesh_path = if let Some(abs_path) = found_path {
                        if let Some(rel) = utils::make_relative_path(&abs_path, omsi_root) {
                            rel
                        } else {
                            eprintln!("WARNING: Could not strip prefix {:?} from {:?}. Fallback used.", omsi_root, abs_path);
                            sco_folder.join(mesh_file).to_string_lossy().replace('/', "\\")
                        }
                    } else {
                        log_missing_file(sco_path, "Collision Mesh", mesh_file, &candidates);
                        sco_folder.join(mesh_file).to_string_lossy().replace('/', "\\")
                    };
                    
                    dependencies.insert(mesh_path.clone());
                    
                    // Extract textures embedded in mesh files
                    let mesh_textures = if mesh_file_lower.ends_with(".o3d") {
                        o3d::extract_o3d_textures(&mesh_path, omsi_root)
                    } else if mesh_file_lower.ends_with(".x") {
                        x::extract_x_textures(&mesh_path, omsi_root)
                    } else {
                        None
                    };
                    
                    if let Some(textures) = mesh_textures {
                        for tex_name in textures {
                            let base_name = if let Some(pos) = tex_name.rfind('.') {
                                &tex_name[..pos]
                            } else {
                                &tex_name
                            };
                            utils::add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                        }
                    }
                }
            }
        }
        
        // Extract CTC folders
        if section == "[ctc]" {
            lines.next(); // Skip variable
            if let Some(folder_line) = lines.next() {
                let folder_rel_path = folder_line.trim();
                if !folder_rel_path.is_empty() {
                    // Try relative to sco folder
                    let folder_path = sco_folder.join(folder_rel_path);
                    let full_folder_path = omsi_root.join(&folder_path);
                    
                    if full_folder_path.exists() && full_folder_path.is_dir() {
                        // Add all files in this folder
                        if let Ok(entries) = std::fs::read_dir(&full_folder_path) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_file() {
                                    // Calculate relative path from omsi root
                                    // We constructed full_folder_path from omsi_root + sco_folder + folder_rel_path
                                    // So we can reconstruct the relative path
                                    let file_name = path.file_name().unwrap();
                                    let rel_path = folder_path.join(file_name);
                                    dependencies.insert(rel_path.to_string_lossy().replace('/', "\\"));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Extract textures from [matl] sections
        if section == "[matl]" || section == "[matl_change]" || section == "[matl_lightmap]" {
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                let tex_file_lower = tex_file.to_lowercase();
                if !tex_file.is_empty() && (tex_file_lower.ends_with(".jpg") || tex_file_lower.ends_with(".jpeg") ||
                    tex_file_lower.ends_with(".bmp") || tex_file_lower.ends_with(".dds") || 
                    tex_file_lower.ends_with(".png") || tex_file_lower.ends_with(".tga")) {
                    
                    // Get base name without extension
                    let base_name = if let Some(pos) = tex_file.rfind('.') {
                        &tex_file[..pos]
                    } else {
                        tex_file
                    };
                    
                    // Find all texture variants (different extensions, night folder, etc.)
                    utils::add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                }
            }
        }

        // Extract textures from [CTCTexture] sections
        if section == "[ctctexture]" {
            // Next line is variable name, skip it
            lines.next();
            // Following line is the texture path
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                let tex_file_lower = tex_file.to_lowercase();
                if !tex_file.is_empty() && (tex_file_lower.ends_with(".jpg") || tex_file_lower.ends_with(".jpeg") ||
                    tex_file_lower.ends_with(".bmp") || tex_file_lower.ends_with(".dds") || 
                    tex_file_lower.ends_with(".png") || tex_file_lower.ends_with(".tga")) {
                    
                    // Get base name without extension
                    let base_name = if let Some(pos) = tex_file.rfind('.') {
                        &tex_file[..pos]
                    } else {
                        tex_file
                    };
                    
                    // Find all texture variants
                    utils::add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                }
            }
        }

        // Extract textures from [tree] sections
        if section == "[tree]" {
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                let tex_file_lower = tex_file.to_lowercase();
                if !tex_file.is_empty() && (tex_file_lower.ends_with(".jpg") || tex_file_lower.ends_with(".jpeg") ||
                    tex_file_lower.ends_with(".bmp") || tex_file_lower.ends_with(".dds") || 
                    tex_file_lower.ends_with(".png") || tex_file_lower.ends_with(".tga")) {
                    
                    // Get base name without extension
                    let base_name = if let Some(pos) = tex_file.rfind('.') {
                        &tex_file[..pos]
                    } else {
                        tex_file
                    };
                    
                    // Find all texture variants
                    utils::add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                }
            }
        }
        
        // Extract environment map textures from [matl_envmap] sections
        if section == "[matl_envmap]" {
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                let tex_file_lower = tex_file.to_lowercase();
                if !tex_file.is_empty() && (tex_file_lower.ends_with(".jpg") || tex_file_lower.ends_with(".jpeg") ||
                    tex_file_lower.ends_with(".bmp") || tex_file_lower.ends_with(".dds") || 
                    tex_file_lower.ends_with(".png") || tex_file_lower.ends_with(".tga")) {
                    
                    // Get base name without extension
                    let base_name = if let Some(pos) = tex_file.rfind('.') {
                        &tex_file[..pos]
                    } else {
                        tex_file
                    };
                    
                    // Find all texture variants
                    utils::add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                }
            }
            // Skip the next line (usually a numeric value like 0.85)
            lines.next();
        }
        
        // Extract scripts from [script] sections
        if section == "[script]" {
            // Next line is count, skip it
            lines.next();
            // Following line is the script path
            if let Some(script_line) = lines.next() {
                let script_file_raw = script_line.trim();
                let script_file = script_file_raw.split(&[';', '#'][..]).next().unwrap_or("").trim();
                
                let script_file_lower = script_file.to_lowercase();
                if !script_file.is_empty() && script_file_lower.ends_with(".osc") {
                    let abs_sco_folder = omsi_root.join(sco_folder);
                    
                    let candidates = vec![
                        abs_sco_folder.join("script"),
                        abs_sco_folder.join("Script"),
                        abs_sco_folder.clone(),
                    ];
                    
                    let mut found_path = None;
                    for candidate in &candidates {
                        if let Some(p) = utils::find_file(candidate, script_file) {
                            found_path = Some(p);
                            break;
                        }
                    }
                    
                    if let Some(abs_path) = found_path {
                        if let Some(rel) = utils::make_relative_path(&abs_path, omsi_root) {
                            dependencies.insert(rel);
                        } else {
                            eprintln!("WARNING: Could not strip prefix {:?} from {:?}. Fallback used.", omsi_root, abs_path);
                            dependencies.insert(sco_folder.join(script_file).to_string_lossy().replace('/', "\\"));
                        }
                    } else {
                        log_missing_file(sco_path, "Script", script_file, &candidates);
                        dependencies.insert(sco_folder.join(script_file).to_string_lossy().replace('/', "\\"));
                    }
                }
            }
        }
        
        // Extract varname lists from [varnamelist] sections
        if section == "[varnamelist]" {
            // Next line is count, skip it
            lines.next();
            // Following line is the varlist path
            if let Some(varlist_line) = lines.next() {
                let varlist_file = varlist_line.trim();
                let varlist_file_lower = varlist_file.to_lowercase();
                if !varlist_file.is_empty() && varlist_file_lower.ends_with(".txt") {
                    let abs_sco_folder = omsi_root.join(sco_folder);
                    let abs_script_folder = abs_sco_folder.join("script");
                    let abs_script_folder_cap = abs_sco_folder.join("Script");
                    
                    // Try to find in script/ folder first, then sco root
                    let found_path = utils::find_file(&abs_script_folder, varlist_file)
                        .or_else(|| utils::find_file(&abs_script_folder_cap, varlist_file))
                        .or_else(|| utils::find_file(&abs_sco_folder, varlist_file));
                    
                    if let Some(abs_path) = found_path {
                        if let Some(rel) = utils::make_relative_path(&abs_path, omsi_root) {
                            dependencies.insert(rel);
                        } else {
                            eprintln!("WARNING: Could not strip prefix {:?} from {:?}. Fallback used.", omsi_root, abs_path);
                            dependencies.insert(sco_folder.join(varlist_file).to_string_lossy().replace('/', "\\"));
                        }
                    } else {
                        // Log missing
                        log_missing_file(sco_path, "Varlist", varlist_file, &[abs_script_folder.clone(), abs_script_folder_cap.clone(), abs_sco_folder.clone()]);
                        // Fallback
                        dependencies.insert(sco_folder.join(varlist_file).to_string_lossy().replace('/', "\\"));
                    }
                }
            }
        }
        
        // Extract sound configs from [sound] sections
        if section == "[sound]" {
            if let Some(sound_line) = lines.next() {
                let sound_file = sound_line.trim();
                let sound_file_lower = sound_file.to_lowercase();
                if !sound_file.is_empty() && sound_file_lower.ends_with(".cfg") {
                    let abs_sco_folder = omsi_root.join(sco_folder);
                    let abs_sound_folder = abs_sco_folder.join("sound");
                    let abs_sound_folder_cap = abs_sco_folder.join("Sound");
                    
                    // Try to find in sound/ folder first, then sco root
                    let found_path = utils::find_file(&abs_sound_folder, sound_file)
                        .or_else(|| utils::find_file(&abs_sound_folder_cap, sound_file))
                        .or_else(|| utils::find_file(&abs_sco_folder, sound_file));
                    
                    let sound_path = if let Some(abs_path) = found_path {
                        if let Some(rel) = utils::make_relative_path(&abs_path, omsi_root) {
                            rel
                        } else {
                            eprintln!("WARNING: Could not strip prefix {:?} from {:?}. Fallback used.", omsi_root, abs_path);
                            sco_folder.join(sound_file).to_string_lossy().replace('/', "\\")
                        }
                    } else {
                        // Log missing
                        log_missing_file(sco_path, "Sound", sound_file, &[abs_sound_folder.clone(), abs_sound_folder_cap.clone(), abs_sco_folder.clone()]);
                        sco_folder.join(sound_file).to_string_lossy().replace('/', "\\")
                    };
                    
                    dependencies.insert(sound_path.clone());
                    
                    // TODO: Extract nested sound dependencies
                }
            }
        }
        
        // Extract passenger cabin configs from [passengercabin] sections
        if section == "[passengercabin]" {
            if let Some(cabin_line) = lines.next() {
                let cabin_file = cabin_line.trim();
                let cabin_file_lower = cabin_file.to_lowercase();
                if !cabin_file.is_empty() && cabin_file_lower.ends_with(".cfg") {
                    let abs_sco_folder = omsi_root.join(sco_folder);
                    let abs_model_folder = abs_sco_folder.join("model");
                    let abs_model_folder_cap = abs_sco_folder.join("Model");
                    
                    // Try to find in model/ folder first (common for cabins), then sco root
                    let found_path = utils::find_file(&abs_model_folder, cabin_file)
                        .or_else(|| utils::find_file(&abs_model_folder_cap, cabin_file))
                        .or_else(|| utils::find_file(&abs_sco_folder, cabin_file));
                    
                    if let Some(abs_path) = found_path {
                        if let Some(rel) = utils::make_relative_path(&abs_path, omsi_root) {
                            dependencies.insert(rel);
                        } else {
                            eprintln!("WARNING: Could not strip prefix {:?} from {:?}. Fallback used.", omsi_root, abs_path);
                            dependencies.insert(sco_folder.join(cabin_file).to_string_lossy().replace('/', "\\"));
                        }
                    } else {
                        // Log missing
                        log_missing_file(sco_path, "Cabin", cabin_file, &[abs_model_folder.clone(), abs_model_folder_cap.clone(), abs_sco_folder.clone()]);
                        dependencies.insert(sco_folder.join(cabin_file).to_string_lossy().replace('/', "\\"));
                    }
                }
            }
        }
    }
    
    // Additional texture detection: search for textures matching the .sco filename
    // For example: Dum_cetkovice4.sco should find Dum_cetkovice4_#low.dds
    if let Some(sco_filename) = Path::new(sco_path).file_stem() {
        if let Some(sco_name) = sco_filename.to_str() {
            search_textures_by_prefix(sco_name, &sco_folder, omsi_root, &mut dependencies);
        }
    }
    
    Some(dependencies)
}

/// Search for textures in Texture folders that match the given prefix (e.g., sco filename)
fn search_textures_by_prefix(prefix: &str, sco_folder: &Path, omsi_root: &Path, dependencies: &mut HashSet<String>) {
    use std::fs;
    
    let texture_extensions = ["jpg", "jpeg", "bmp", "dds", "png", "tga"];
    
    // Try multiple texture folder locations
    let search_paths = vec![
        sco_folder.join("texture"),  // Sceneryobjects\XYZ\texture\
        sco_folder.to_path_buf(),    // Sceneryobjects\XYZ\
        Path::new("Texture").to_path_buf(), // Global Texture\
    ];
    
    for search_path in search_paths {
        let full_search_path = omsi_root.join(&search_path);
        
        if !full_search_path.exists() || !full_search_path.is_dir() {
            continue;
        }
        
        // Search in main folder
        if let Ok(entries) = fs::read_dir(&full_search_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(filename) = entry_path.file_name().and_then(|n| n.to_str()) {
                        // Check if filename contains the prefix
                        let filename_lower = filename.to_lowercase();
                        let prefix_lower = prefix.to_lowercase();
                        
                        if filename_lower.starts_with(&prefix_lower) {
                            // Check if it has a texture extension
                            for ext in &texture_extensions {
                                if filename_lower.ends_with(&format!(".{}", ext)) {
                                    let file_path = search_path.join(filename);
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
                                    
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Also search in subfolders (night, alpha, winter, etc.)
        let seasonal_folders = ["night", "Night", "alpha", "Alpha", "winter", "Winter", "WinterSnow", "wintersnow", "spring", "Spring", "fall", "Fall"];
        
        for subfolder in &seasonal_folders {
            let seasonal_path = search_path.join(subfolder);
            let full_seasonal_path = omsi_root.join(&seasonal_path);
            
            if full_seasonal_path.exists() && full_seasonal_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&full_seasonal_path) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            if let Some(filename) = entry_path.file_name().and_then(|n| n.to_str()) {
                                let filename_lower = filename.to_lowercase();
                                let prefix_lower = prefix.to_lowercase();
                                
                                if filename_lower.starts_with(&prefix_lower) {
                                    for ext in &texture_extensions {
                                        if filename_lower.ends_with(&format!(".{}", ext)) {
                                            let file_path = seasonal_path.join(filename);
                                            let path_str = file_path.to_string_lossy().replace('/', "\\");
                                            dependencies.insert(path_str.clone());
                                            
                                            let cfg_path = format!("{}.cfg", path_str);
                                            let surf_path = format!("{}.surf", path_str);
                                            
                                            if omsi_root.join(&cfg_path).exists() {
                                                dependencies.insert(cfg_path);
                                            }
                                            if omsi_root.join(&surf_path).exists() {
                                                dependencies.insert(surf_path);
                                            }
                                            
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Log missing file to stderr
fn log_missing_file(sco_path: &str, file_type: &str, filename: &str, searched_paths: &[std::path::PathBuf]) {
    let paths_str = searched_paths.iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
        
    eprintln!("WARNING: Missing {} '{}' in '{}'. Searched: {}", file_type, filename, sco_path, paths_str);
}
