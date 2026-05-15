use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProviderStatusKind {
    Ready,
    NeedsConfiguration,
    InvalidConfig,
    UnsupportedProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderStatus {
    pub status: ModelProviderStatusKind,
    pub message: String,
    pub config_path: String,
    pub created_template: bool,
    pub provider: Option<String>,
    pub api_url: Option<String>,
    pub model_name: Option<String>,
    pub has_api_key: bool,
    pub api_key_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    Assistant,
    Engineer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageIntent {
    Status,
    Diagnosis,
    FollowUp,
    Answer,
    Advisory,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStage {
    Idle,
    Analyzing,
    AwaitingAnswer,
    UpdatingPreview,
    AdvisoryComplete,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfProcessingMode {
    DirectToModel,
    LocalPreprocess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    Addition,
    Rewrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub intent: MessageIntent,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisclosureSection {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub id: String,
    pub kind: AnnotationKind,
    pub section_id: String,
    pub excerpt: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisclosureDocument {
    pub version: u32,
    pub sections: Vec<DisclosureSection>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoSettings {
    pub pdf_mode: PdfProcessingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionActions {
    pub can_submit: bool,
    pub can_skip: bool,
    pub can_download: bool,
    pub can_terminate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub section_id: Option<String>,
    pub level: ValidationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryMetadata {
    pub attempt_count: u8,
    pub max_attempts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProgress {
    pub current_question_index: usize,
    pub total_questions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub session_id: String,
    pub stage: SessionStage,
    pub source_draft: String,
    pub messages: Vec<ChatMessage>,
    pub disclosure: DisclosureDocument,
    pub settings: DemoSettings,
    pub actions: SessionActions,
    pub progress: SessionProgress,
    pub validation_issues: Vec<ValidationIssue>,
    pub retry_metadata: RetryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDemoSessionRequest {
    pub settings: DemoSettings,
    pub source_draft: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerRequest {
    pub session: SessionSnapshot,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequest {
    pub session: SessionSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub session: SessionSnapshot,
    pub settings: DemoSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub session: SessionSnapshot,
    pub include_answer_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Ready,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResponse {
    pub status: DownloadStatus,
    pub file_name: String,
    pub message: String,
    pub included_answer_history: bool,
    pub validation_issue_count: usize,
}