use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::model_config::ModelProviderConfig;
use crate::models::{
    Annotation, AnnotationKind, ChatMessage, DemoSettings, DisclosureDocument, DisclosureSection,
    MessageIntent, MessageRole, RetryMetadata, SessionActions, SessionProgress, SessionSnapshot,
    SessionStage, StartDemoSessionRequest,
};
use crate::validation::validate_disclosure_document;

const STATIC_TIMESTAMP: &str = "2026-05-15T09:00:00+08:00";

const SECTION_DEFINITIONS: [(&str, &str); 10] = [
    ("title", "一、发明名称"),
    ("field", "二、技术领域"),
    ("background", "三、背景技术及现有缺陷"),
    ("purpose", "四、发明目的"),
    ("technical_solution", "五、技术方案"),
    ("benefits", "六、有益效果"),
    ("figures", "七、附图说明"),
    ("implementation", "八、具体实施方式"),
    ("layout", "🔍 专利挖掘与布局建议"),
    ("questions", "🛠️ 需工程师补齐的关键改进建议"),
];

pub enum UserTurn {
    Answer(String),
    Skip,
}

pub struct QwenClient {
    client: Client,
    api_key: String,
    endpoint: String,
    model: String,
}

#[derive(Serialize)]
struct QwenChatRequest<'a> {
    model: &'a str,
    messages: Vec<QwenMessage<'a>>,
    temperature: f32,
    max_tokens: u16,
}

#[derive(Serialize)]
struct QwenMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct QwenChatResponse {
    choices: Vec<QwenChoice>,
}

#[derive(Deserialize)]
struct QwenChoice {
    message: QwenAssistantMessage,
}

#[derive(Deserialize)]
struct QwenAssistantMessage {
    content: String,
}

#[derive(Deserialize)]
struct QwenErrorEnvelope {
    error: QwenErrorBody,
}

#[derive(Deserialize)]
struct QwenErrorBody {
    message: String,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AiDisclosurePayload {
    diagnosis_summary: String,
    #[serde(default)]
    follow_up_question: Option<String>,
    #[serde(default)]
    completeness_advisory: Option<String>,
    sections: AiDisclosureSections,
}

#[derive(Debug, Deserialize)]
struct AiDisclosureSections {
    #[serde(default)]
    title: String,
    #[serde(default)]
    field: String,
    #[serde(default)]
    background: String,
    #[serde(default)]
    purpose: String,
    #[serde(default)]
    technical_solution: String,
    #[serde(default)]
    benefits: String,
    #[serde(default)]
    figures: String,
    #[serde(default)]
    implementation: String,
    #[serde(default)]
    layout: String,
    #[serde(default)]
    questions: String,
}

struct GeneratedDisclosure {
    diagnosis_summary: String,
    follow_up_question: Option<String>,
    completeness_advisory: Option<String>,
    disclosure: DisclosureDocument,
    attempts: u8,
    reached_retry_limit: bool,
}

impl QwenClient {
    pub fn from_config(config: ModelProviderConfig) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .map_err(|error| format!("初始化 Qwen HTTP 客户端失败：{error}"))?;

        Ok(Self {
            client,
            api_key: config.api_key.trim().to_string(),
            endpoint: config.api_url.trim().to_string(),
            model: config.model_name.trim().to_string(),
        })
    }

    pub fn start_session(
        &self,
        request: StartDemoSessionRequest,
    ) -> Result<SessionSnapshot, String> {
        let source_draft = request.source_draft.trim().to_string();
        let generated = self.generate_disclosure(&source_draft, None, None)?;
        Ok(build_live_session(
            "qwen-live-session",
            request.settings,
            source_draft,
            None,
            None,
            generated,
        ))
    }

    pub fn submit_answer(
        &self,
        session: SessionSnapshot,
        answer: String,
    ) -> Result<SessionSnapshot, String> {
        let settings = session.settings.clone();
        let session_id = session.session_id.clone();
        let source_draft = session.source_draft.clone();
        let generated = self.generate_disclosure(
            &source_draft,
            Some(&session),
            Some(&UserTurn::Answer(answer.clone())),
        )?;
        Ok(build_live_session(
            &session_id,
            settings,
            source_draft,
            Some(session),
            Some(UserTurn::Answer(answer)),
            generated,
        ))
    }

    pub fn skip_question(&self, session: SessionSnapshot) -> Result<SessionSnapshot, String> {
        let settings = session.settings.clone();
        let session_id = session.session_id.clone();
        let source_draft = session.source_draft.clone();
        let generated = self.generate_disclosure(&source_draft, Some(&session), Some(&UserTurn::Skip))?;
        Ok(build_live_session(
            &session_id,
            settings,
            source_draft,
            Some(session),
            Some(UserTurn::Skip),
            generated,
        ))
    }

    fn generate_disclosure(
        &self,
        source_draft: &str,
        previous: Option<&SessionSnapshot>,
        turn: Option<&UserTurn>,
    ) -> Result<GeneratedDisclosure, String> {
        let mut last_payload = None;
        let mut last_disclosure = None;

        for attempt in 1..=3 {
            let prompt = build_prompt(source_draft, previous, turn, attempt);
            let payload = self.call_qwen(&prompt)?;
            let disclosure = disclosure_from_payload(previous, &payload);
            let issues = validate_disclosure_document(&disclosure);
            let has_error = issues.iter().any(|issue| matches!(issue.level, crate::models::ValidationLevel::Error));

            last_disclosure = Some(disclosure.clone());
            last_payload = Some(payload);

            if !has_error {
                return Ok(GeneratedDisclosure {
                    diagnosis_summary: last_payload
                        .as_ref()
                        .map(|current| current.diagnosis_summary.clone())
                        .unwrap_or_default(),
                    follow_up_question: sanitized_optional(
                        last_payload
                            .as_ref()
                            .and_then(|current| current.follow_up_question.clone()),
                    ),
                    completeness_advisory: sanitized_optional(
                        last_payload
                            .as_ref()
                            .and_then(|current| current.completeness_advisory.clone()),
                    ),
                    disclosure,
                    attempts: attempt,
                    reached_retry_limit: false,
                });
            }
        }

        let fallback_payload = last_payload.ok_or_else(|| "Qwen 未返回可解析内容。".to_string())?;
        let fallback_disclosure = last_disclosure.ok_or_else(|| "Qwen 未生成交底书内容。".to_string())?;

        Ok(GeneratedDisclosure {
            diagnosis_summary: fallback_payload.diagnosis_summary,
            follow_up_question: sanitized_optional(fallback_payload.follow_up_question),
            completeness_advisory: sanitized_optional(fallback_payload.completeness_advisory),
            disclosure: fallback_disclosure,
            attempts: 3,
            reached_retry_limit: true,
        })
    }

    fn call_qwen(&self, user_prompt: &str) -> Result<AiDisclosurePayload, String> {
        let request = QwenChatRequest {
            model: &self.model,
            messages: vec![
                QwenMessage {
                    role: "system",
                    content: build_system_prompt(),
                },
                QwenMessage {
                    role: "user",
                    content: user_prompt.to_string(),
                },
            ],
            temperature: 0.4,
            max_tokens: 2800,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .map_err(|error| format!("Qwen 请求失败：{error}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            let message = serde_json::from_str::<QwenErrorEnvelope>(&body)
                .map(|parsed| match parsed.error.code {
                    Some(code) => format!("{} ({})", parsed.error.message, code),
                    None => parsed.error.message,
                })
                .unwrap_or(body);

            return Err(format!("Qwen 接口返回 {}：{}", status, message));
        }

        let response_body: QwenChatResponse = response
            .json()
            .map_err(|error| format!("Qwen 响应解析失败：{error}"))?;

        let content = response_body
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| "Qwen 未返回 choices[0].message.content。".to_string())?;

        parse_ai_payload(&content)
    }
}

fn build_live_session(
    session_id: &str,
    settings: DemoSettings,
    source_draft: String,
    previous: Option<SessionSnapshot>,
    turn: Option<UserTurn>,
    generated: GeneratedDisclosure,
) -> SessionSnapshot {
    let mut messages = previous
        .as_ref()
        .map(|session| session.messages.clone())
        .unwrap_or_default();

    let completed_turns_before = previous
        .as_ref()
        .map(|session| session.progress.current_question_index)
        .unwrap_or(0);

    if previous.is_none() {
        messages.push(message(
            "live-start",
            MessageRole::System,
            MessageIntent::Status,
            "已通过配置文件指定的 Qwen 模型实时分析演示草稿。当前会话会继续沿用该模型配置执行追问与预览更新。",
        ));
    }

    let completed_turns = match turn.as_ref() {
        Some(UserTurn::Answer(answer)) => {
            messages.push(message(
                &format!("answer-{}", completed_turns_before + 1),
                MessageRole::Engineer,
                MessageIntent::Answer,
                answer,
            ));
            messages.push(message(
                &format!("live-status-{}", completed_turns_before + 1),
                MessageRole::System,
                MessageIntent::Status,
                "已通过 Qwen 实时更新右侧交底书预览。",
            ));
            completed_turns_before + 1
        }
        Some(UserTurn::Skip) => {
            messages.push(message(
                &format!("skip-{}", completed_turns_before + 1),
                MessageRole::System,
                MessageIntent::Status,
                "已跳过当前追问，并要求 Qwen 在保留待补充标记的前提下生成下一步建议。",
            ));
            completed_turns_before + 1
        }
        None => completed_turns_before,
    };

    messages.push(message(
        &format!("diagnosis-{}", completed_turns.max(1)),
        MessageRole::Assistant,
        MessageIntent::Diagnosis,
        &generated.diagnosis_summary,
    ));

    if generated.reached_retry_limit {
        messages.push(message(
            &format!("validation-warning-{}", completed_turns.max(1)),
            MessageRole::System,
            MessageIntent::Status,
            "Qwen 已返回最新版本，但格式校验未完全通过，当前结果已保留并显示校验提示。",
        ));
    }

    let has_follow_up = generated
        .follow_up_question
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    if has_follow_up {
        messages.push(message(
            &format!("follow-up-{}", completed_turns + 1),
            MessageRole::Assistant,
            MessageIntent::FollowUp,
            generated.follow_up_question.as_deref().unwrap_or_default(),
        ));
    } else {
        messages.push(message(
            &format!("advisory-{}", completed_turns.max(1)),
            MessageRole::Assistant,
            MessageIntent::Advisory,
            generated
                .completeness_advisory
                .as_deref()
                .unwrap_or("当前交底书信息已较为完整，可以继续补充，也可以直接下载当前版本。"),
        ));
    }

    let total_questions = (completed_turns + usize::from(has_follow_up)).max(1);
    let stage = if has_follow_up {
        SessionStage::AwaitingAnswer
    } else {
        SessionStage::AdvisoryComplete
    };

    SessionSnapshot {
        session_id: session_id.to_string(),
        stage,
        source_draft,
        messages,
        disclosure: generated.disclosure.clone(),
        settings,
        actions: if has_follow_up {
            active_actions()
        } else {
            advisory_actions()
        },
        progress: SessionProgress {
            current_question_index: completed_turns,
            total_questions,
        },
        validation_issues: validate_disclosure_document(&generated.disclosure),
        retry_metadata: RetryMetadata {
            attempt_count: generated.attempts,
            max_attempts: 3,
        },
    }
}

fn build_system_prompt() -> String {
    r#"你是一位资深专利工程师与专利代理人协作专家。你的任务是把工程师草稿和补充回答，整理为符合 PatentScribe AI V1 结构约束的技术交底书。

必须严格输出 JSON 对象，不得输出 Markdown 代码块、解释文字或额外前后缀。JSON 结构必须为：
{
  "diagnosis_summary": "字符串",
  "follow_up_question": "字符串，若无需继续追问则输出空字符串",
  "completeness_advisory": "字符串，若仍需继续追问则输出空字符串",
  "sections": {
    "title": "一、发明名称 内容",
    "field": "二、技术领域 内容",
    "background": "三、背景技术及现有缺陷 内容",
    "purpose": "四、发明目的 内容",
    "technical_solution": "五、技术方案 内容",
    "benefits": "六、有益效果 内容，必须是 Markdown 表格，表头固定为 | 维度 | 技术突破点 | 效果与价值体现 |",
    "figures": "七、附图说明 内容",
    "implementation": "八、具体实施方式 内容",
    "layout": "专项输出，必须是 Markdown 表格，表头固定为 | 布局方向 | 保护策略 | 建议权利要求架构 |",
    "questions": "专项输出，必须使用 Q1/Q2 问答式"
  }
}

硬约束：
- 信息缺失时只能写【待工程师补充】，不得虚构技术事实。
- 第五部分“技术方案”使用工程师口吻，不要堆砌法律术语。
- follow_up_question 和 completeness_advisory 只能二选一有内容；若仍需追问，follow_up_question 必须是可直接回复的单条问题。
- 具体实施方式必须尽量结合已有回答补全参数、触发条件、控制逻辑。
"#.to_string()
}

fn build_prompt(
    source_draft: &str,
    previous: Option<&SessionSnapshot>,
    turn: Option<&UserTurn>,
    attempt: u8,
) -> String {
    let conversation_history = previous
        .map(extract_conversation_history)
        .filter(|history| !history.is_empty())
        .unwrap_or_else(|| "无历史问答。".to_string());

    let current_disclosure = previous
        .map(|session| format_disclosure(&session.disclosure))
        .unwrap_or_else(|| "无上一版交底书。".to_string());

    let latest_turn = match turn {
        Some(UserTurn::Answer(answer)) => format!("本轮工程师补充：{}", answer),
        Some(UserTurn::Skip) => {
            "本轮工程师选择跳过当前追问，请保留待补充标记并继续生成下一条更聚焦的追问。".to_string()
        }
        None => "这是初次分析，请基于原始草稿生成诊断、交底书和第一条追问。".to_string(),
    };

    let retry_guidance = if attempt > 1 {
        format!(
            "这是第 {} 次生成尝试。请重点修正上一次输出中的结构问题，确保 8 个固定章节和 2 个专项输出全部出现，且表格与 Q1/Q2 格式正确。",
            attempt
        )
    } else {
        String::new()
    };

    format!(
        "原始工程草稿：\n{}\n\n当前交底书版本：\n{}\n\n历史问答：\n{}\n\n{}\n\n{}",
        source_draft.trim(),
        current_disclosure,
        conversation_history,
        latest_turn,
        retry_guidance,
    )
}

fn extract_conversation_history(session: &SessionSnapshot) -> String {
    let mut lines = Vec::new();

    for message in &session.messages {
        match (message.role.clone(), message.intent.clone()) {
            (MessageRole::Assistant, MessageIntent::FollowUp) => {
                lines.push(format!("AI 追问：{}", message.content));
            }
            (MessageRole::Engineer, MessageIntent::Answer) => {
                lines.push(format!("工程师回答：{}", message.content));
            }
            (MessageRole::System, MessageIntent::Status)
                if message.content.contains("跳过当前追问") =>
            {
                lines.push("工程师操作：跳过上一条追问。".to_string());
            }
            _ => {}
        }
    }

    if lines.is_empty() {
        "无历史问答。".to_string()
    } else {
        lines.join("\n")
    }
}

fn disclosure_from_payload(
    previous: Option<&SessionSnapshot>,
    payload: &AiDisclosurePayload,
) -> DisclosureDocument {
    let section_values = payload.sections.as_map();
    let sections = SECTION_DEFINITIONS
        .iter()
        .map(|(id, title)| DisclosureSection {
            id: (*id).to_string(),
            title: (*title).to_string(),
            content: section_values
                .get(*id)
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string(),
        })
        .collect::<Vec<_>>();

    let version = previous
        .map(|session| session.disclosure.version + 1)
        .unwrap_or(1);

    let provisional = DisclosureDocument {
        version,
        sections,
        annotations: Vec::new(),
    };

    let annotations = derive_annotations(previous.map(|session| &session.disclosure), &provisional);

    DisclosureDocument {
        annotations,
        ..provisional
    }
}

fn derive_annotations(
    previous: Option<&DisclosureDocument>,
    next: &DisclosureDocument,
) -> Vec<Annotation> {
    let previous_sections = previous
        .map(|document| {
            document
                .sections
                .iter()
                .map(|section| (section.id.as_str(), section.content.as_str()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    next.sections
        .iter()
        .filter_map(|section| {
            let previous_content = previous_sections.get(section.id.as_str()).copied();
            if previous_content == Some(section.content.as_str()) {
                return None;
            }

            let kind = match previous_content {
                None => AnnotationKind::Addition,
                Some(previous) if previous.trim().is_empty() => AnnotationKind::Addition,
                Some(previous)
                    if section.content.contains(previous)
                        || section.content.len() > previous.len() + 24 =>
                {
                    AnnotationKind::Addition
                }
                Some(_) => AnnotationKind::Rewrite,
            };

            let note = match kind {
                AnnotationKind::Addition => "基于 Qwen 分析结果补充了新的工程或专利表达。",
                AnnotationKind::Rewrite => "基于 Qwen 分析结果重写了原有描述，使其更适合交底书结构。",
            };

            let excerpt = match previous_content {
                Some(previous) if section.content.starts_with(previous) => {
                    snippet(section.content.trim_start_matches(previous))
                }
                _ => snippet(&section.content),
            };

            if excerpt.is_empty() {
                return None;
            }

            Some(Annotation {
                id: format!("ann-{}-v{}", section.id, next.version),
                kind,
                section_id: section.id.clone(),
                excerpt,
                note: note.to_string(),
            })
        })
        .collect()
}

fn snippet(content: &str) -> String {
    content
        .trim()
        .chars()
        .take(42)
        .collect::<String>()
        .trim()
        .to_string()
}

fn parse_ai_payload(content: &str) -> Result<AiDisclosurePayload, String> {
    let trimmed = content.trim();
    let json_payload = if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed.to_string()
    } else {
        let start = trimmed
            .find('{')
            .ok_or_else(|| format!("Qwen 返回内容不是 JSON：{}", trimmed))?;
        let end = trimmed
            .rfind('}')
            .ok_or_else(|| format!("Qwen 返回内容不是 JSON：{}", trimmed))?;
        trimmed[start..=end].to_string()
    };

    serde_json::from_str::<AiDisclosurePayload>(&json_payload)
        .map_err(|error| format!("Qwen JSON 解析失败：{}；原始内容：{}", error, trimmed))
}

fn sanitized_optional(value: Option<String>) -> Option<String> {
    value.and_then(|current| {
        let trimmed = current.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn format_disclosure(disclosure: &DisclosureDocument) -> String {
    disclosure
        .sections
        .iter()
        .map(|section| format!("{}\n{}", section.title, section.content))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn message(id: &str, role: MessageRole, intent: MessageIntent, content: &str) -> ChatMessage {
    ChatMessage {
        id: id.to_string(),
        role,
        intent,
        content: content.to_string(),
        timestamp: STATIC_TIMESTAMP.to_string(),
    }
}

fn active_actions() -> SessionActions {
    SessionActions {
        can_submit: true,
        can_skip: true,
        can_download: true,
        can_terminate: true,
    }
}

fn advisory_actions() -> SessionActions {
    SessionActions {
        can_submit: true,
        can_skip: false,
        can_download: true,
        can_terminate: true,
    }
}

impl AiDisclosureSections {
    fn as_map(&self) -> BTreeMap<&'static str, String> {
        BTreeMap::from([
            ("title", self.title.clone()),
            ("field", self.field.clone()),
            ("background", self.background.clone()),
            ("purpose", self.purpose.clone()),
            ("technical_solution", self.technical_solution.clone()),
            ("benefits", self.benefits.clone()),
            ("figures", self.figures.clone()),
            ("implementation", self.implementation.clone()),
            ("layout", self.layout.clone()),
            ("questions", self.questions.clone()),
        ])
    }
}