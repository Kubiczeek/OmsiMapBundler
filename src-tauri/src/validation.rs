use std::path::Path;
use crate::types::ValidationResult;

// Validate that the map folder contains all required files
pub fn validate_map_folder(map_folder: String) -> ValidationResult {
    let path = Path::new(&map_folder);
    
    if !path.exists() {
        return ValidationResult {
            valid: false,
            missing_files: vec![],
            error: Some("Map folder does not exist".to_string()),
        };
    }
    
    let required_files = vec![
        "global.cfg",
        "ailists.cfg",
        "drivers.txt",
        "parklist_p.txt",
    ];
    
    let mut missing = Vec::new();
    
    for file in required_files {
        if !path.join(file).exists() {
            missing.push(file.to_string());
        }
    }
    
    ValidationResult {
        valid: missing.is_empty(),
        missing_files: missing,
        error: None,
    }
}
