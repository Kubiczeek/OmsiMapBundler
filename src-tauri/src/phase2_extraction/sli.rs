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
                    utils::add_texture_variants(base_name, &sli_folder, omsi_root, &mut dependencies);
                }
            }
        }
    }
    
    Some(dependencies)
}
