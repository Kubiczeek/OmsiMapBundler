use std::path::Path;
use std::collections::HashSet;
use crate::phase2_extraction::utils;

/// Extract dependencies from a generic .cfg file (used by humans, etc.)
pub fn extract_cfg_dependencies(cfg_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_cfg_path = omsi_root.join(cfg_path);
    
    if !full_cfg_path.exists() {
        return None;
    }
    
    let mut dependencies = HashSet::new();
    let cfg_folder = Path::new(cfg_path).parent().unwrap_or(Path::new(""));
    
    // Read .cfg file with Windows-1252 encoding
    let cfg_content = match utils::read_file_windows1252(&full_cfg_path) {
        Some(content) => content,
        None => return None,
    };
    
    let mut lines = cfg_content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Extract textures from [texture] sections (common in human configs)
        if trimmed == "[texture]" {
            if let Some(tex_line) = lines.next() {
                let tex_file = tex_line.trim();
                if !tex_file.is_empty() {
                    // Try relative to cfg folder
                    let tex_path = cfg_folder.join(tex_file);
                    let tex_path_str = tex_path.to_string_lossy().replace('/', "\\");
                    
                    // Check if it exists
                    if omsi_root.join(&tex_path_str).exists() {
                        dependencies.insert(tex_path_str);
                    } else {
                        // Try in Texture subfolder
                        let tex_path2 = cfg_folder.join("Texture").join(tex_file);
                        let tex_path2_str = tex_path2.to_string_lossy().replace('/', "\\");
                        if omsi_root.join(&tex_path2_str).exists() {
                            dependencies.insert(tex_path2_str);
                        }
                    }
                }
            }
        }
    }
    
    Some(dependencies)
}

/// Extract dependencies from a sound config file
pub fn extract_sound_cfg_dependencies(cfg_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_cfg_path = omsi_root.join(cfg_path);
    
    if !full_cfg_path.exists() {
        return None;
    }
    
    let mut dependencies = HashSet::new();
    let cfg_folder = Path::new(cfg_path).parent().unwrap_or(Path::new(""));
    
    // Read sound config file with Windows-1252 encoding
    let cfg_content = match utils::read_file_windows1252(&full_cfg_path) {
        Some(content) => content,
        None => return None,
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
