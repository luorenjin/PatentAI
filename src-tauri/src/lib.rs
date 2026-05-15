mod commands;
mod model_config;
mod mock_session;
mod models;
mod qwen_session;
mod session_service;
mod validation;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::start_demo_session,
            commands::submit_demo_answer,
            commands::skip_demo_question,
            commands::update_demo_settings,
            commands::terminate_demo_session,
            commands::download_demo_snapshot,
            commands::get_model_provider_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
