use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

/// Extract all dependencies from a .sco file
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_sceneryobject_dependencies(sco_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_sco_path = omsi_root.join(sco_path);
    
    if !full_sco_path.exists() {
        println!("Sceneryobject file not found: {:?}", full_sco_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .sco file itself
    dependencies.insert(sco_path.to_string());
    
    // Read .sco file with Windows-1252 encoding
    let sco_content = match File::open(&full_sco_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file);
            let mut content = String::new();
            match decoder.read_to_string(&mut content) {
                Ok(_) => content,
                Err(e) => {
                    println!("Failed to decode {}: {}", sco_path, e);
                    return None;
                }
            }
        }
        Err(e) => {
            println!("Failed to open {}: {}", sco_path, e);
            return None;
        }
    };
    
    let sco_folder = Path::new(sco_path).parent().unwrap_or(Path::new(""));
    let mut lines = sco_content.lines();
    
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Extract mesh files from [mesh] sections
        if trimmed == "[mesh]" {
            if let Some(mesh_line) = lines.next() {
                let mesh_file = mesh_line.trim();
                if !mesh_file.is_empty() && mesh_file.ends_with(".o3d") {
                    // Try multiple locations for the mesh file
                    // Option 1: relative to sco folder + model subfolder
                    let option1 = sco_folder.join("model").join(mesh_file);
                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                    let test1 = omsi_root.join(&option1_str);
                    
                    // Option 2: relative to sco folder directly
                    let option2 = sco_folder.join(mesh_file);
                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                    let test2 = omsi_root.join(&option2_str);
                    
                    let mesh_path = if test1.exists() {
                        option1_str.clone()
                    } else if test2.exists() {
                        option2_str.clone()
                    } else {
                        // Fallback: use as-is
                        mesh_file.replace('/', "\\")
                    };
                    
                    dependencies.insert(mesh_path.clone());
                    
                    // Extract textures embedded in .o3d file
                    if let Some(o3d_textures) = extract_o3d_textures(&mesh_path, omsi_root) {
                        for tex_name in o3d_textures {
                            // Get base name without extension
                            let base_name = if let Some(pos) = tex_name.rfind('.') {
                                &tex_name[..pos]
                            } else {
                                &tex_name
                            };
                            
                            // Find all texture variants
                            add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                        }
                    }
                }
            }
        }
        
        // Extract textures from [matl] sections
        if trimmed == "[matl]" {
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
                    add_texture_variants(base_name, &sco_folder, omsi_root, &mut dependencies);
                }
            }
        }
        
        // Extract scripts from [script] sections
        if trimmed == "[script]" {
            // Next line is count, skip it
            lines.next();
            // Following line is the script path
            if let Some(script_line) = lines.next() {
                let script_file = script_line.trim();
                if !script_file.is_empty() && script_file.ends_with(".osc") {
                    // Try multiple locations
                    // Option 1: relative to sco folder + script subfolder
                    let option1 = sco_folder.join("script").join(script_file);
                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                    let test1 = omsi_root.join(&option1_str);
                    
                    // Option 2: relative to sco folder directly
                    let option2 = sco_folder.join(script_file);
                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                    let test2 = omsi_root.join(&option2_str);
                    
                    if test1.exists() {
                        dependencies.insert(option1_str);
                    } else if test2.exists() {
                        dependencies.insert(option2_str);
                    } else {
                        // Fallback: use as-is (might be full path already)
                        let fallback = script_file.replace('/', "\\");
                        dependencies.insert(fallback);
                    }
                }
            }
        }
        
        // Extract varname lists from [varnamelist] sections
        if trimmed == "[varnamelist]" {
            // Next line is count, skip it
            lines.next();
            // Following line is the varlist path
            if let Some(varlist_line) = lines.next() {
                let varlist_file = varlist_line.trim();
                if !varlist_file.is_empty() && varlist_file.ends_with(".txt") {
                    // Try multiple locations
                    // Option 1: relative to sco folder + script subfolder
                    let option1 = sco_folder.join("script").join(varlist_file);
                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                    let test1 = omsi_root.join(&option1_str);
                    
                    // Option 2: relative to sco folder directly
                    let option2 = sco_folder.join(varlist_file);
                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                    let test2 = omsi_root.join(&option2_str);
                    
                    if test1.exists() {
                        dependencies.insert(option1_str);
                    } else if test2.exists() {
                        dependencies.insert(option2_str);
                    } else {
                        // Fallback: use as-is
                        let fallback = varlist_file.replace('/', "\\");
                        dependencies.insert(fallback);
                    }
                }
            }
        }
        
        // Extract sound configs from [sound] sections
        if trimmed == "[sound]" {
            if let Some(sound_line) = lines.next() {
                let sound_file = sound_line.trim();
                if !sound_file.is_empty() && sound_file.ends_with(".cfg") {
                    // Try multiple locations
                    // Option 1: relative to sco folder + sound subfolder
                    let option1 = sco_folder.join("sound").join(sound_file);
                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                    let test1 = omsi_root.join(&option1_str);
                    
                    // Option 2: relative to sco folder directly
                    let option2 = sco_folder.join(sound_file);
                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                    let test2 = omsi_root.join(&option2_str);
                    
                    let sound_path = if test1.exists() {
                        option1_str
                    } else if test2.exists() {
                        option2_str
                    } else {
                        // Fallback: use as-is
                        sound_file.replace('/', "\\")
                    };
                    
                    dependencies.insert(sound_path.clone());
                    
                    // Extract nested sound dependencies
                    if let Some(sound_deps) = extract_sound_config_dependencies(&sound_path, omsi_root) {
                        dependencies.extend(sound_deps);
                    }
                }
            }
        }
    }
    
    Some(dependencies)
}

/// Extract dependencies from a sound config file
fn extract_sound_config_dependencies(cfg_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_cfg_path = omsi_root.join(cfg_path);
    
    if !full_cfg_path.exists() {
        return None;
    }
    
    let mut dependencies = HashSet::new();
    let cfg_folder = Path::new(cfg_path).parent().unwrap_or(Path::new(""));
    
    // Read sound config file with Windows-1252 encoding
    let cfg_content = match File::open(&full_cfg_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file);
            let mut content = String::new();
            match decoder.read_to_string(&mut content) {
                Ok(_) => content,
                Err(_) => return None,
            }
        }
        Err(_) => return None,
    };
    
    // Parse sound files - look for .wav files
    for line in cfg_content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.ends_with(".wav") {
            // Try multiple locations for .wav files
            // Option 1: relative to cfg folder + sound subfolder
            let option1 = cfg_folder.join("sound").join(trimmed);
            let option1_str = option1.to_string_lossy().replace('/', "\\");
            let test1 = omsi_root.join(&option1_str);
            
            // Option 2: relative to cfg folder directly
            let option2 = cfg_folder.join(trimmed);
            let option2_str = option2.to_string_lossy().replace('/', "\\");
            let test2 = omsi_root.join(&option2_str);
            
            if test1.exists() {
                dependencies.insert(option1_str);
            } else if test2.exists() {
                dependencies.insert(option2_str);
            } else {
                // Fallback: use as-is
                let fallback = trimmed.replace('/', "\\");
                dependencies.insert(fallback);
            }
        }
    }
    
    Some(dependencies)
}

/// Helper function to find all texture variants with the same base name
fn add_texture_variants(base_name: &str, sco_folder: &Path, omsi_root: &Path, dependencies: &mut HashSet<String>) {
    use std::fs;
    
    let texture_extensions = ["jpg", "bmp", "dds", "png", "tga"];
    
    // Try multiple base locations
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
        for ext in &texture_extensions {
            let file_name = format!("{}.{}", base_name, ext);
            let file_path = search_path.join(&file_name);
            let full_file_path = omsi_root.join(&file_path);
            
            if full_file_path.exists() {
                let path_str = file_path.to_string_lossy().replace('/', "\\");
                dependencies.insert(path_str);
            }
            
            // Also check case-insensitive match
            let file_name_lower = format!("{}.{}", base_name.to_lowercase(), ext);
            if file_name.to_lowercase() != file_name_lower {
                let file_path_ci = search_path.join(&file_name_lower);
                let full_file_path_ci = omsi_root.join(&file_path_ci);
                
                if full_file_path_ci.exists() {
                    let path_str = file_path_ci.to_string_lossy().replace('/', "\\");
                    dependencies.insert(path_str);
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
                            dependencies.insert(path_str);
                        }
                        
                        // Case-insensitive check
                        let file_name_lower = format!("{}.{}", base_name.to_lowercase(), ext);
                        if file_name.to_lowercase() != file_name_lower {
                            let file_path_ci = search_path.join(&subfolder_name).join(&file_name_lower);
                            let full_file_path_ci = omsi_root.join(&file_path_ci);
                            
                            if full_file_path_ci.exists() {
                                let path_str = file_path_ci.to_string_lossy().replace('/', "\\");
                                dependencies.insert(path_str);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract texture references from .o3d binary file
fn extract_o3d_textures(o3d_path: &str, omsi_root: &Path) -> Option<Vec<String>> {
    use std::fs;
    use std::io::Read;
    
    let full_o3d_path = omsi_root.join(o3d_path);
    
    if !full_o3d_path.exists() {
        return None;
    }
    
    // Read binary file
    let mut file = match fs::File::open(&full_o3d_path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    
    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        return None;
    }
    
    let mut textures = Vec::new();
    let texture_extensions = [".bmp", ".tga", ".dds", ".jpg", ".png"];
    
    // Search for null-terminated strings that look like texture filenames
    let mut i = 0;
    while i < buffer.len() {
        // Look for printable ASCII characters
        if buffer[i] >= 32 && buffer[i] < 127 {
            let start = i;
            let mut end = i;
            
            // Find the end of the string (null terminator or non-ASCII)
            while end < buffer.len() && buffer[end] >= 32 && buffer[end] < 127 {
                end += 1;
            }
            
            if end > start {
                if let Ok(text) = String::from_utf8(buffer[start..end].to_vec()) {
                    // Check if it looks like a texture filename
                    let text_lower = text.to_lowercase();
                    for ext in &texture_extensions {
                        if text_lower.ends_with(ext) {
                            // Found a texture reference
                            textures.push(text.clone());
                            break;
                        }
                    }
                }
            }
            
            i = end + 1;
        } else {
            i += 1;
        }
    }
    
    if textures.is_empty() {
        None
    } else {
        Some(textures)
    }
}
