use crate::session_service;
use crate::models::{
    AnswerRequest, DownloadRequest, DownloadResponse, ModelProviderStatus, SessionRequest,
    SessionSnapshot, StartDemoSessionRequest, UpdateSettingsRequest,
};

#[tauri::command]
pub fn start_demo_session(
    app: tauri::AppHandle,
    request: StartDemoSessionRequest,
) -> Result<SessionSnapshot, String> {
    session_service::start_session(&app, request)
}

#[tauri::command]
pub fn submit_demo_answer(
    app: tauri::AppHandle,
    request: AnswerRequest,
) -> Result<SessionSnapshot, String> {
    session_service::submit_answer(&app, request)
}

#[tauri::command]
pub fn skip_demo_question(
    app: tauri::AppHandle,
    request: SessionRequest,
) -> Result<SessionSnapshot, String> {
    session_service::skip_question(&app, request)
}

#[tauri::command]
pub fn update_demo_settings(request: UpdateSettingsRequest) -> SessionSnapshot {
    session_service::update_settings(request)
}

#[tauri::command]
pub fn terminate_demo_session(request: SessionRequest) -> SessionSnapshot {
    session_service::terminate_session(request)
}

#[tauri::command]
pub fn download_demo_snapshot(request: DownloadRequest) -> DownloadResponse {
    session_service::download_snapshot(request)
}

#[tauri::command]
pub fn get_model_provider_status(
    app: tauri::AppHandle,
) -> Result<ModelProviderStatus, String> {
    session_service::get_model_provider_status(&app)
}