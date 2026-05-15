use tauri::AppHandle;

use crate::model_config::{self, ModelProviderConfigState};
use crate::mock_session;
use crate::models::{
    AnswerRequest, DownloadRequest, DownloadResponse, ModelProviderStatus, SessionRequest,
    SessionSnapshot, StartDemoSessionRequest, UpdateSettingsRequest,
};
use crate::qwen_session::QwenClient;

pub fn start_session(
    app: &AppHandle,
    mut request: StartDemoSessionRequest,
) -> Result<SessionSnapshot, String> {
    request.source_draft = request.source_draft.trim().to_string();
    if request.source_draft.is_empty() {
        return Err("请先输入待分析的交底草稿，再启动会话。".to_string());
    }

    match resolve_runtime(app)? {
        RuntimeTarget::Live(client) => client.start_session(request),
        RuntimeTarget::Mock(notice) => {
            Ok(with_mock_notice(mock_session::start_session(request), &notice))
        }
    }
}

pub fn submit_answer(app: &AppHandle, request: AnswerRequest) -> Result<SessionSnapshot, String> {
    match resolve_runtime(app)? {
        RuntimeTarget::Live(client) => client.submit_answer(request.session, request.answer),
        RuntimeTarget::Mock(notice) => Ok(with_mock_notice(mock_session::submit_answer(
            request.session,
            request.answer,
        ), &notice)),
    }
}

pub fn skip_question(app: &AppHandle, request: SessionRequest) -> Result<SessionSnapshot, String> {
    match resolve_runtime(app)? {
        RuntimeTarget::Live(client) => client.skip_question(request.session),
        RuntimeTarget::Mock(notice) => {
            Ok(with_mock_notice(mock_session::skip_question(request.session), &notice))
        }
    }
}

pub fn update_settings(request: UpdateSettingsRequest) -> SessionSnapshot {
    mock_session::update_settings(request)
}

pub fn terminate_session(request: SessionRequest) -> SessionSnapshot {
    mock_session::terminate_session(request)
}

pub fn download_snapshot(request: DownloadRequest) -> DownloadResponse {
    mock_session::download_snapshot(request)
}

pub fn get_model_provider_status(app: &AppHandle) -> Result<ModelProviderStatus, String> {
    model_config::get_status(app)
}

enum RuntimeTarget {
    Live(QwenClient),
    Mock(String),
}

fn resolve_runtime(app: &AppHandle) -> Result<RuntimeTarget, String> {
    let config_state = ModelProviderConfigState::load(app)?;

    match config_state.config {
        Some(config) => match config.provider_slug().as_str() {
            provider if model_config::is_supported_provider(provider) => {
                Ok(RuntimeTarget::Live(QwenClient::from_config(config)?))
            }
            provider => Err(format!(
                "模型配置文件中的 PROVIDER={} 暂不支持。当前仅支持 qwen / dashscope 兼容链路。配置文件：{}",
                provider,
                config_state.config_path.display()
            )),
        },
        None => Ok(RuntimeTarget::Mock(config_state.fallback_notice())),
    }
}

fn with_mock_notice(mut session: SessionSnapshot, notice: &str) -> SessionSnapshot {
    let has_notice = session
        .messages
        .iter()
        .any(|message| message.id == "mock-backend-notice");

    if !has_notice {
        session.messages.insert(
            0,
            crate::models::ChatMessage {
                id: "mock-backend-notice".to_string(),
                role: crate::models::MessageRole::System,
                intent: crate::models::MessageIntent::Status,
                content: notice.to_string(),
                timestamp: "2026-05-15T09:00:00+08:00".to_string(),
            },
        );
    }

    session
}