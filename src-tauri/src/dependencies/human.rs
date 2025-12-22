use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use walkdir::WalkDir;

/// Helper function to add all textures from a CTC texture folder
fn add_textures_from_ctc_folder(base_path: &str, cfg_folder: &Path, omsi_root: &Path, dependencies: &mut HashSet<String>) {
    let human_base = cfg_folder.parent().unwrap_or(Path::new(""));
    
    // Try to find textures in: Humans\aXYZ\texture\<subfolder from base_path>
    // base_path is like "Texture\woman01", we want to extract "woman01"
    let subfolder = base_path.replace("Texture\\", "").replace("Texture/", "");
    
    // Option 1: Check in Humans\aXYZ\texture\woman01\
    let local_texture_folder = human_base.join("texture").join(&subfolder);
    let full_local_path = omsi_root.join(&local_texture_folder);
    
    if full_local_path.exists() && full_local_path.is_dir() {
        for entry in WalkDir::new(&full_local_path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "jpg" || ext_str == "bmp" || ext_str == "dds" || 
                   ext_str == "png" || ext_str == "tga" {
                    // Build relative path
                    let rel_path = local_texture_folder.join(entry.file_name());
                    let rel_str = rel_path.to_string_lossy().replace('/', "\\");
                    dependencies.insert(rel_str);
                }
            }
        }
    }
}

/// Extract all dependencies from a .hum file
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_human_dependencies(hum_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_hum_path = omsi_root.join(hum_path);
    
    if !full_hum_path.exists() {
        println!("Human file not found: {:?}", full_hum_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .hum file itself
    dependencies.insert(hum_path.to_string());
    
    // Read .hum file with Windows-1252 encoding
    let hum_content = match File::open(&full_hum_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file);
            let mut content = String::new();
            match decoder.read_to_string(&mut content) {
                Ok(_) => content,
                Err(e) => {
                    println!("Failed to decode {}: {}", hum_path, e);
                    return None;
                }
            }
        }
        Err(e) => {
            println!("Failed to open {}: {}", hum_path, e);
            return None;
        }
    };
    
    // Parse [model] section to find the .cfg file
    let mut lines = hum_content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        if trimmed == "[model]" {
            // Next line contains the model .cfg path
            if let Some(cfg_line) = lines.next() {
                let cfg_path = cfg_line.trim();
                
                if !cfg_path.is_empty() {
                    // Build full path relative to human folder
                    let human_folder = Path::new(hum_path).parent().unwrap_or(Path::new(""));
                    let full_cfg_path = human_folder.join(cfg_path);
                    let cfg_path_str = full_cfg_path.to_string_lossy().replace('/', "\\");
                    
                    dependencies.insert(cfg_path_str.clone());
                    
                    // Extract dependencies from the .cfg file
                    if let Some(cfg_deps) = extract_cfg_dependencies(&cfg_path_str, omsi_root) {
                        dependencies.extend(cfg_deps);
                    }
                }
            }
        }
    }
    
    Some(dependencies)
}

/// Extract dependencies from a human model .cfg file
fn extract_cfg_dependencies(cfg_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_cfg_path = omsi_root.join(cfg_path);
    
    if !full_cfg_path.exists() {
        println!("Config file not found: {:?}", full_cfg_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    let cfg_folder = Path::new(cfg_path).parent().unwrap_or(Path::new(""));
    
    // Read .cfg file with Windows-1252 encoding
    let cfg_content = match File::open(&full_cfg_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(file);
            let mut content = String::new();
            match decoder.read_to_string(&mut content) {
                Ok(_) => content,
                Err(e) => {
                    println!("Failed to decode {}: {}", cfg_path, e);
                    return None;
                }
            }
        }
        Err(e) => {
            println!("Failed to open {}: {}", cfg_path, e);
            return None;
        }
    };
    
    // First pass: find the texture base path from [CTC] section and add all textures from that folder
    let mut texture_base_path: Option<String> = None;
    let mut lines = cfg_content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "[CTC]" {
            // Skip next line (usually "Colorscheme" or empty)
            if let Some(next_line) = lines.next() {
                let next_trimmed = next_line.trim();
                // If it's not "Colorscheme" or empty, it might be the path
                if next_trimmed.is_empty() || next_trimmed.eq_ignore_ascii_case("Colorscheme") {
                    // The actual path is on the next line
                    if let Some(tex_path_line) = lines.next() {
                        let tex_path = tex_path_line.trim();
                        if !tex_path.is_empty() && !tex_path.chars().all(|c| c.is_numeric()) {
                            texture_base_path = Some(tex_path.to_string());
                            
                            // Add all textures from this folder
                            add_textures_from_ctc_folder(&tex_path, &cfg_folder, omsi_root, &mut dependencies);
                            break;
                        }
                    }
                } else if !next_trimmed.chars().all(|c| c.is_numeric()) {
                    // The path might be on this line directly
                    texture_base_path = Some(next_trimmed.to_string());
                    
                    // Add all textures from this folder
                    add_textures_from_ctc_folder(&next_trimmed, &cfg_folder, omsi_root, &mut dependencies);
                    break;
                }
            }
        }
    }
    
    // Second pass: extract mesh and texture files
    let mut lines = cfg_content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Extract mesh files from [mesh] sections - include ALL meshes
        if trimmed == "[mesh]" {
            if let Some(mesh_line) = lines.next() {
                let mesh_file = mesh_line.trim();
                if !mesh_file.is_empty() && mesh_file.ends_with(".o3d") {
                    let full_mesh_path = cfg_folder.join(mesh_file);
                    let mesh_path_str = full_mesh_path.to_string_lossy().replace('/', "\\");
                    dependencies.insert(mesh_path_str);
                }
            }
        }
        
        // Extract textures from [CTCTexture] sections
        if trimmed == "[CTCTexture]" {
            // Skip next line (farbschema or similar)
            lines.next();
            // Next line contains the texture filename
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                if !tex_file.is_empty() && (tex_file.ends_with(".jpg") || 
                    tex_file.ends_with(".bmp") || tex_file.ends_with(".dds") || 
                    tex_file.ends_with(".png") || tex_file.ends_with(".tga")) {
                    
                    // Build full texture path - try multiple locations
                    let texture_path = if let Some(base_path) = &texture_base_path {
                        // Get parent of model folder (e.g., Humans\aXYZ)
                        let human_base = cfg_folder.parent().unwrap_or(Path::new(""));
                        
                        // Option 1: Humans\aXYZ\texture\subfolder\file.jpg
                        let option1 = human_base.join("texture").join(base_path).join(tex_file);
                        let option1_str = option1.to_string_lossy().replace('/', "\\");
                        let test1 = omsi_root.join(&option1_str);
                        
                        // Option 2: Humans\aXYZ\texture\file.jpg (directly in texture folder)
                        let option2 = human_base.join("texture").join(tex_file);
                        let option2_str = option2.to_string_lossy().replace('/', "\\");
                        let test2 = omsi_root.join(&option2_str);
                        
                        // Option 3: Texture\subfolder\file.jpg (global)
                        let option3 = format!("{}\\{}", base_path, tex_file);
                        
                        if test1.exists() {
                            option1_str
                        } else if test2.exists() {
                            option2_str
                        } else {
                            option3
                        }
                    } else {
                        // Fallback: just use Texture\ folder
                        format!("Texture\\{}", tex_file)
                    };
                    dependencies.insert(texture_path);
                }
            }
        }
    }
    
    Some(dependencies)
}
