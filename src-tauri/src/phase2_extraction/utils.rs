use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::WINDOWS_1252;

/// Find a file in a directory case-insensitively
pub fn find_file(base_path: &Path, filename: &str) -> Option<PathBuf> {
    // First try direct match (fastest)
    let direct_path = base_path.join(filename);
    if direct_path.exists() && direct_path.is_file() {
        return Some(direct_path);
    }
    
    // Try case-insensitive search
    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.eq_ignore_ascii_case(filename) {
                    return Some(entry.path());
                }
            }
        }
    }
    
    None
}

/// Create a relative path from a full path and a root path, handling case-insensitivity on Windows
pub fn make_relative_path(full_path: &Path, root_path: &Path) -> Option<String> {
    // Try standard strip_prefix first
    if let Ok(rel) = full_path.strip_prefix(root_path) {
        return Some(rel.to_string_lossy().replace('/', "\\"));
    }
    
    // Try case-insensitive matching
    let full_str = full_path.to_string_lossy();
    let root_str = root_path.to_string_lossy();
    
    if full_str.to_lowercase().starts_with(&root_str.to_lowercase()) {
        let rel_str = &full_str[root_str.len()..];
        let rel_str = rel_str.trim_start_matches('\\').trim_start_matches('/');
        return Some(rel_str.replace('/', "\\"));
    }
    
    None
}

/// Read a file with Windows-1252 encoding (with manual BOM detection)
pub fn read_file_windows1252(path: &Path) -> Option<String> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open {:?}: {}", path, e);
            return None;
        }
    };
    
    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        eprintln!("Failed to read {:?}: {}", path, e);
        return None;
    }
    
    // Check for BOMs
    if buffer.len() >= 2 && buffer[0] == 0xFF && buffer[1] == 0xFE {
        // UTF-16 LE
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(&buffer[2..]);
        return Some(cow.into_owned());
    } else if buffer.len() >= 2 && buffer[0] == 0xFE && buffer[1] == 0xFF {
        // UTF-16 BE
        let (cow, _, _) = encoding_rs::UTF_16BE.decode(&buffer[2..]);
        return Some(cow.into_owned());
    } else if buffer.len() >= 3 && buffer[0] == 0xEF && buffer[1] == 0xBB && buffer[2] == 0xBF {
        // UTF-8
        let (cow, _, _) = encoding_rs::UTF_8.decode(&buffer[3..]);
        return Some(cow.into_owned());
    }
    
    // Fallback to Windows-1252
    let (cow, _, _) = WINDOWS_1252.decode(&buffer);
    Some(cow.into_owned())
}

/// Helper function to find all texture variants with the same base name
pub fn add_texture_variants(base_name: &str, base_folder: &Path, omsi_root: &Path, dependencies: &mut HashSet<String>) {
    use std::fs;
    
    let texture_extensions = ["jpg", "jpeg", "bmp", "dds", "png", "tga"];
    let seasonal_folders = ["night", "Night", "alpha", "Alpha", "winter", "Winter", "WinterSnow", "wintersnow", "spring", "Spring", "fall", "Fall"];
    
    // Try multiple base locations
    let search_paths = vec![
        base_folder.join("texture"),  // Object\texture\
        base_folder.to_path_buf(),    // Object\
        Path::new("Texture").to_path_buf(), // Global Texture\
    ];
    
    for search_path in search_paths {
        let full_search_path = omsi_root.join(&search_path);
        
        if !full_search_path.exists() || !full_search_path.is_dir() {
            continue;
        }

        // Optimization: Read directory content once to avoid many exists() calls
        let mut dir_files = HashSet::new();
        if let Ok(entries) = fs::read_dir(&full_search_path) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    dir_files.insert(name.to_lowercase());
                }
            }
        }
        
        // Search in main folder
        for ext in &texture_extensions {
            let file_name = format!("{}.{}", base_name, ext);
            let file_name_lower = file_name.to_lowercase();
            
            if dir_files.contains(&file_name_lower) {
                // File exists (case-insensitive check passed)
                // We need to find the actual case-sensitive name if possible, or just use constructed one
                // Since Windows is case-insensitive, using the constructed one usually works for copying
                // But for exact matching, we might want the real name.
                // For now, let's try to find the real name from the dir_files if we stored it properly?
                // No, we stored lowercase.
                // Let's just use the constructed path, but we know it exists.
                
                // Wait, we need the actual filename on disk for Linux/Case-sensitive systems?
                // OMSI is Windows only, so maybe not critical, but good practice.
                // Let's just use the constructed path.
                
                let file_path = search_path.join(&file_name);
                let path_str = file_path.to_string_lossy().replace('/', "\\");
                dependencies.insert(path_str.clone());
                
                // Also add .cfg and .surf files if they exist
                // We can check dir_files for these too
                let cfg_name = format!("{}.cfg", file_name);
                if dir_files.contains(&cfg_name.to_lowercase()) {
                    let cfg_path = format!("{}.cfg", path_str);
                    dependencies.insert(cfg_path);
                }
                
                let surf_name = format!("{}.surf", file_name);
                if dir_files.contains(&surf_name.to_lowercase()) {
                    let surf_path = format!("{}.surf", path_str);
                    dependencies.insert(surf_path);
                }
            }
        }
        
        // Search in seasonal/variant subfolders (night, alpha, winter, etc.)
        for subfolder in &seasonal_folders {
            let seasonal_path = search_path.join(subfolder);
            let full_seasonal_path = omsi_root.join(&seasonal_path);
            
            if full_seasonal_path.exists() && full_seasonal_path.is_dir() {
                // Optimization: Read directory content once
                let mut seasonal_files = HashSet::new();
                if let Ok(entries) = fs::read_dir(&full_seasonal_path) {
                    for entry in entries.flatten() {
                        if let Ok(name) = entry.file_name().into_string() {
                            seasonal_files.insert(name.to_lowercase());
                        }
                    }
                }

                for ext in &texture_extensions {
                    let file_name = format!("{}.{}", base_name, ext);
                    let file_name_lower = file_name.to_lowercase();
                    
                    if seasonal_files.contains(&file_name_lower) {
                        let file_path = seasonal_path.join(&file_name);
                        let path_str = file_path.to_string_lossy().replace('/', "\\");
                        dependencies.insert(path_str.clone());
                        
                        // Also add .cfg and .surf files if they exist
                        let cfg_name = format!("{}.cfg", file_name);
                        if seasonal_files.contains(&cfg_name.to_lowercase()) {
                            let cfg_path = format!("{}.cfg", path_str);
                            dependencies.insert(cfg_path);
                        }
                        
                        let surf_name = format!("{}.surf", file_name);
                        if seasonal_files.contains(&surf_name.to_lowercase()) {
                            let surf_path = format!("{}.surf", path_str);
                            dependencies.insert(surf_path);
                        }
                    }
                }
            }
        }
        
        // Also search any other subfolders we haven't explicitly checked
        // This part was very slow because it recursed into all subfolders
        // Let's optimize it to only check 1 level deep and only if it looks like a texture folder
        if let Ok(entries) = fs::read_dir(&full_search_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let subfolder_name = entry_path.file_name().unwrap().to_string_lossy().to_string();
                    
                    // Skip if we already checked this folder
                    if seasonal_folders.contains(&subfolder_name.as_str()) {
                        continue;
                    }
                    
                    // Optimization: Read directory content once
                    let mut sub_files = HashSet::new();
                    if let Ok(sub_entries) = fs::read_dir(&entry_path) {
                        for sub_entry in sub_entries.flatten() {
                            if let Ok(name) = sub_entry.file_name().into_string() {
                                sub_files.insert(name.to_lowercase());
                            }
                        }
                    }
                    
                    for ext in &texture_extensions {
                        let file_name = format!("{}.{}", base_name, ext);
                        let file_name_lower = file_name.to_lowercase();
                        
                        if sub_files.contains(&file_name_lower) {
                            let file_path = search_path.join(&subfolder_name).join(&file_name);
                            let path_str = file_path.to_string_lossy().replace('/', "\\");
                            dependencies.insert(path_str.clone());
                            
                            // Also add .cfg and .surf files if they exist
                            let cfg_name = format!("{}.cfg", file_name);
                            if sub_files.contains(&cfg_name.to_lowercase()) {
                                let cfg_path = format!("{}.cfg", path_str);
                                dependencies.insert(cfg_path);
                            }
                            
                            let surf_name = format!("{}.surf", file_name);
                            if sub_files.contains(&surf_name.to_lowercase()) {
                                let surf_path = format!("{}.surf", path_str);
                                dependencies.insert(surf_path);
                            }
                        }
                    }
                }
            }
        }
    }
}


/// Extract texture names from binary content
pub fn extract_textures_from_binary(buffer: &[u8]) -> Option<Vec<String>> {
    let mut textures = Vec::new();
    let texture_extensions: &[&[u8]] = &[b".bmp", b".tga", b".dds", b".jpg", b".jpeg", b".png", b".BMP", b".TGA", b".DDS", b".JPG", b".JPEG", b".PNG"];
    
    // Search for texture file extensions and then go backwards to find the filename
    for ext in texture_extensions {
        let ext_len = ext.len();
        let mut i = 0;
        
        while i + ext_len <= buffer.len() {
            // Check if we found an extension
            if &buffer[i..i + ext_len] == *ext {
                // Found extension, now go backwards to find the start of the filename
                let mut start = i;
                let mut found_valid_chars = false;
                
                // Go back while we find valid filename characters
                while start > 0 {
                    let prev_idx = start - 1;
                    let c = buffer[prev_idx];
                    
                    // Valid filename characters: letters, digits, underscore, hyphen, dot, backslash, forward slash
                    if (c >= b'A' && c <= b'Z') || 
                       (c >= b'a' && c <= b'z') ||
                       (c >= b'0' && c <= b'9') ||
                       c == b'_' || c == b'-' || c == b'.' || c == b'\\' || c == b'/' || c == b'#' {
                        start = prev_idx;
                        found_valid_chars = true;
                    } else {
                        // Found invalid character, stop here
                        break;
                    }
                }
                
                // Extract the filename
                let end = i + ext_len;
                if end > start && start < buffer.len() && found_valid_chars {
                    if let Ok(filename) = String::from_utf8(buffer[start..end].to_vec()) {
                        // Clean up the filename - remove any leading invalid characters
                        let cleaned = filename
                            .chars()
                            .skip_while(|c| !c.is_alphanumeric() && *c != '_')
                            .collect::<String>();
                        
                        // Extract just the filename without path
                        let final_name = if let Some(pos) = cleaned.rfind('\\') {
                            &cleaned[pos + 1..]
                        } else if let Some(pos) = cleaned.rfind('/') {
                            &cleaned[pos + 1..]
                        } else {
                            &cleaned
                        };
                        
                        // Validate that it looks like a reasonable filename
                        if final_name.len() > ext_len && !textures.contains(&final_name.to_string()) {
                            let first_char = final_name.chars().next().unwrap();
                            if first_char.is_alphanumeric() || first_char == '_' {
                                textures.push(final_name.to_string());
                            }
                        }
                    }
                }
                
                // Move past this extension
                i += ext_len;
            } else {
                i += 1;
            }
        }
    }
    
    if textures.is_empty() {
        None
    } else {
        Some(textures)
    }
}
