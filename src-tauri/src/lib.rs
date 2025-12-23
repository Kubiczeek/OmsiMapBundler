// Module declarations
mod types;
mod validation;
mod extraction;
mod bundling;
mod utils;
mod dependencies;

// Re-export types for Tauri commands
use types::{ValidationResult, DependencyResult, BundleRequest, BundleResult};
use tauri::{async_runtime, Emitter};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Clone)]
struct ProgressPayload {
    message: String,
    progress: f32,
}

// Tauri command wrappers
#[tauri::command]
async fn validate_map_folder(map_folder: String) -> ValidationResult {
    async_runtime::spawn_blocking(move || validation::validate_map_folder(map_folder))
        .await
        .unwrap_or_else(|_| ValidationResult { valid: false, missing_files: vec![], error: Some("Validation task failed".into()) })
}

#[tauri::command]
async fn extract_dependencies(map_folder: String) -> DependencyResult {
    async_runtime::spawn_blocking(move || extraction::extract_dependencies(map_folder))
        .await
        .unwrap_or_else(|_| DependencyResult {
            sceneryobjects: vec![],
            splines: vec![],
            textures: vec![],
            humans: vec![],
            vehicles: vec![],
            money_systems: vec![],
            ticket_packs: vec![],
            tile_maps: vec![],
            error: Some("Dependency extraction task failed".into()),
        })
}

#[tauri::command]
async fn create_bundle(app_handle: tauri::AppHandle, request: BundleRequest) -> BundleResult {
    let handle = app_handle.clone();
    async_runtime::spawn_blocking(move || {
        let cb: Arc<bundling::ProgressCallback> = Arc::new(Box::new(move |message: &str, progress: f32| {
            let _ = handle.emit(
                "bundle-progress",
                ProgressPayload {
                    message: message.to_string(),
                    progress,
                },
            );
        }));

        bundling::create_bundle(request, Some(cb))
    })
    .await
    .unwrap_or_else(|_| BundleResult {
        success: false,
        output_path: None,
        error: Some("Bundle task failed".into()),
    })
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
