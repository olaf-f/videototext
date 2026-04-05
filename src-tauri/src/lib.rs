pub mod commands;
pub mod error;
pub mod models;
pub mod services;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::ai::structure_text_with_deepseek,
            commands::export::normalize_export_filename,
            commands::export::save_text_export,
            commands::settings::load_settings,
            commands::settings::save_deepseek_api_key,
            commands::settings::save_settings,
            commands::imports::validate_image_url_content_type,
            commands::imports::import_image_from_url,
            commands::ocr::validate_ocr_assets,
            commands::ocr::run_ocr
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SmartOCR Pro workspace scaffold");
}
