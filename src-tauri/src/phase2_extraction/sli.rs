use std::path::Path;
use std::collections::HashSet;
use crate::phase2_extraction::utils;

/// Extract all dependencies from a .sli file
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_sli_dependencies(sli_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_sli_path = omsi_root.join(sli_path);
    
    if !full_sli_path.exists() {
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .sli file itself
    dependencies.insert(sli_path.to_string());
    
    // Read .sli file with Windows-1252 encoding
    let sli_content = match utils::read_file_windows1252(&full_sli_path) {
        Some(content) => content,
        None => return None,
    };
    
    let sli_folder = Path::new(sli_path).parent().unwrap_or(Path::new(""));
    let mut lines = sli_content.lines();
    
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Extract textures from [texture] sections
        if trimmed == "[texture]" {
            if let Some(tex_line) = lines.next() {
                let tex_file_raw = tex_line.trim();
                // Handle comments
                let tex_file = tex_file_raw.split(&[';', '#'][..]).next().unwrap_or("").trim();
                let tex_file_lower = tex_file.to_lowercase();
                
                if !tex_file.is_empty() && (tex_file_lower.ends_with(".jpg") || 
                    tex_file_lower.ends_with(".bmp") || tex_file_lower.ends_with(".dds") || 
                    tex_file_lower.ends_with(".png") || tex_file_lower.ends_with(".tga")) {
                    
                    // Check if it's a path with separators
                    if tex_file.contains('\\') || tex_file.contains('/') {
                        let path_obj = Path::new(tex_file);
                        let file_name = path_obj.file_name().unwrap().to_str().unwrap();
                        let dir_path = path_obj.parent().unwrap();
                        
                        let base_name_no_ext = if let Some(pos) = file_name.rfind('.') {
                            &file_name[..pos]
                        } else {
                            file_name
                        };

                        // 1. Try as path relative to OMSI root
                        utils::add_texture_variants(base_name_no_ext, dir_path, omsi_root, &mut dependencies);
                        
                        // 2. Try as path relative to SLI folder
                        let rel_dir = sli_folder.join(dir_path);
                        utils::add_texture_variants(base_name_no_ext, &rel_dir, omsi_root, &mut dependencies);
                    } else {
                        // Standard behavior
                        let base_name = if let Some(pos) = tex_file.rfind('.') {
                            &tex_file[..pos]
                        } else {
                            tex_file
                        };
                        utils::add_texture_variants(base_name, &sli_folder, omsi_root, &mut dependencies);
                    }
                }
            }
        }
    }
    
    Some(dependencies)
}
