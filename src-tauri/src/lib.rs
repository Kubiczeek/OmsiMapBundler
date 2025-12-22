// Module declarations
mod types;
mod validation;
mod extraction;
mod bundling;
mod utils;

// Re-export types for Tauri commands
use types::{ValidationResult, DependencyResult, BundleRequest, BundleResult};

// Tauri command wrappers
#[tauri::command]
fn validate_map_folder(map_folder: String) -> ValidationResult {
    validation::validate_map_folder(map_folder)
}

#[tauri::command]
fn extract_dependencies(map_folder: String) -> DependencyResult {
    extraction::extract_dependencies(map_folder)
}

#[tauri::command]
fn create_bundle(request: BundleRequest) -> BundleResult {
    bundling::create_bundle(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            validate_map_folder,
            extract_dependencies,
            create_bundle
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
