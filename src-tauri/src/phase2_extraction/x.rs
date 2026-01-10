use std::path::Path;
use std::fs;
use crate::phase2_extraction::utils;

/// Extract texture references from .x (DirectX mesh) file
pub fn extract_x_textures(x_path: &str, omsi_root: &Path) -> Option<Vec<String>> {
    let full_x_path = omsi_root.join(x_path);
    
    if !full_x_path.exists() {
        return None;
    }
    
    // Read file bytes first
    let buffer = match fs::read(&full_x_path) {
        Ok(b) => b,
        Err(_) => return None,
    };
    
    // Try to decode as text (DirectX .x files can be text-based)
    let content = if let Ok(s) = String::from_utf8(buffer.clone()) {
        s
    } else {
        // Try Windows-1252 for legacy files
        let (cow, _, had_errors) = encoding_rs::WINDOWS_1252.decode(&buffer);
        if had_errors {
            // If decoding fails, treat as binary
            return utils::extract_textures_from_binary(&buffer);
        }
        cow.into_owned()
    };
    
    // Parse text-based .x file for texture references
    let mut textures = Vec::new();
    
    // Look for TextureFilename sections in DirectX .x format
    // Example: TextureFilename { "texture.bmp"; }
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Check for TextureFilename keyword
        if trimmed.contains("TextureFilename") {
            // Extract texture name from quotes
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let tex_name = &trimmed[start + 1..start + 1 + end];
                    if !tex_name.is_empty() && !textures.contains(&tex_name.to_string()) {
                        // Clean up path separators
                        let cleaned = tex_name.replace('/', "\\");
                        // Extract just the filename if it contains path
                        let filename = if let Some(pos) = cleaned.rfind('\\') {
                            &cleaned[pos + 1..]
                        } else {
                            &cleaned
                        };
                        textures.push(filename.to_string());
                    }
                }
            }
        }
    }
    
    if textures.is_empty() {
        None
    } else {
        Some(textures)
    }
}
