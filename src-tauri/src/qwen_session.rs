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
    max_tokens: u32,
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
    title: serde_json::Value,
    #[serde(default)]
    field: serde_json::Value,
    #[serde(default)]
    background: serde_json::Value,
    #[serde(default)]
    purpose: serde_json::Value,
    #[serde(default)]
    technical_solution: serde_json::Value,
    #[serde(default)]
    benefits: serde_json::Value,
    #[serde(default)]
    figures: serde_json::Value,
    #[serde(default)]
    implementation: serde_json::Value,
    #[serde(default)]
    layout: serde_json::Value,
    #[serde(default)]
    questions: serde_json::Value,
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
            temperature: 0.3,
            max_tokens: 49152,
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

【输出格式 — 最高优先级】
你的整个回复必须是且仅是一个合法 JSON 对象。禁止任何前缀、后缀、Markdown 代码块标记（如 ```json）、解释文字。第一个字符必须是 {，最后一个字符必须是 }。

JSON 结构固定如下（10 个 sections 字段全部必填，不得遗漏任何一个）：
{
  "diagnosis_summary": "对当前草稿的整体诊断，2-4 句话",
  "follow_up_question": "若仍需追问则填写一条可直接回复的问题，否则填空字符串",
  "completeness_advisory": "若不再追问则填写完整度评语，否则填空字符串",
  "sections": {
    "title": "发明名称，一句话",
    "field": "技术领域描述",
    "background": "背景技术及现有缺陷",
    "purpose": "发明目的",
    "technical_solution": "技术方案，使用工程师口吻",
    "benefits": "有益效果，必须是 Markdown 表格，表头固定为 | 维度 | 技术突破点 | 效果与价值体现 |",
    "figures": "附图说明",
    "implementation": "具体实施方式，尽量结合已有回答补全参数",
    "layout": "专利挖掘与布局建议，必须是 Markdown 表格，表头固定为 | 布局方向 | 保护策略 | 建议权利要求架构 |",
    "questions": "需工程师补齐的关键改进建议，必须使用 Q1/Q2 问答式"
  }
}

硬约束：
1. sections 中 10 个字段全部必须有实质内容（至少两句话），绝对不能为空字符串。
2. 信息缺失时只能写【待工程师补充】，不得虚构技术事实。
3. 第五部分"技术方案"使用工程师口吻，不要堆砌法律术语。
4. follow_up_question 和 completeness_advisory 只能二选一有内容。
5. 每个 section 内容要精练，避免冗长，确保总输出不超过 3500 tokens。
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

    // On retries, omit the full disclosure to shorten the prompt and reduce
    // the chance of hitting the token limit on the response.
    let current_disclosure = if attempt > 1 {
        "（省略上一版全文以节省空间，请基于你已知的上下文重新生成完整 10 个 sections）".to_string()
    } else {
        previous
            .map(|session| format_disclosure(&session.disclosure))
            .unwrap_or_else(|| "无上一版交底书。".to_string())
    };

    let latest_turn = match turn {
        Some(UserTurn::Answer(answer)) => format!("本轮工程师补充：{}", answer),
        Some(UserTurn::Skip) => {
            "本轮工程师选择跳过当前追问，请保留待补充标记并继续生成下一条更聚焦的追问。".to_string()
        }
        None => "这是初次分析，请基于原始草稿生成诊断、交底书和第一条追问。".to_string(),
    };

    let retry_guidance = if attempt > 1 {
        format!(
            "⚠️ 这是第 {} 次生成尝试。上一次输出存在结构问题（可能是 JSON 不完整、sections 字段为空、或缺少表格/Q 格式）。请务必：\n\
             1. 输出必须是完整合法的 JSON\n\
             2. sections 全部 10 个字段都必须有内容\n\
             3. benefits 和 layout 必须包含指定表头的 Markdown 表格\n\
             4. questions 必须使用 Q1/Q2 格式\n\
             5. 每段内容精练，总输出控制在 3000 tokens 以内",
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

    // Build a lookup of previous section content so we can fall back to it
    // when the AI returns an empty string for a section (commonly caused by
    // token-limit truncation or formatting instability).
    let previous_content: std::collections::BTreeMap<&str, &str> = previous
        .map(|session| {
            session
                .disclosure
                .sections
                .iter()
                .map(|s| (s.id.as_str(), s.content.as_str()))
                .collect()
        })
        .unwrap_or_default();

    let sections = SECTION_DEFINITIONS
        .iter()
        .map(|(id, title)| {
            let ai_content = section_values
                .get(*id)
                .cloned()
                .unwrap_or_default();
            let trimmed = ai_content.trim();

            // If the AI returned an empty section but we had content before,
            // preserve the previous version so the preview never regresses to
            // showing only titles.
            let final_content = if trimmed.is_empty() {
                previous_content
                    .get(*id)
                    .copied()
                    .unwrap_or_default()
                    .to_string()
            } else {
                trimmed.to_string()
            };

            DisclosureSection {
                id: (*id).to_string(),
                title: (*title).to_string(),
                content: final_content,
            }
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

    // Step 1: Strip markdown code fences that models frequently wrap around JSON.
    let stripped = strip_markdown_code_block(trimmed);

    // Step 2: Extract the outermost balanced JSON object using brace-depth
    // matching. This is more reliable than the previous first-'{' / last-'}'
    // approach which broke on trailing text or unbalanced content.
    let json_payload = extract_balanced_json(&stripped)
        .ok_or_else(|| {
            let preview: String = trimmed.chars().take(200).collect();
            format!("Qwen 返回内容中未找到完整的 JSON 对象：{}", preview)
        })?;

    // Step 3: Try to parse directly first.
    if let Ok(payload) = serde_json::from_str::<AiDisclosurePayload>(&json_payload) {
        return Ok(payload);
    }

    // Step 4: If direct parse fails, attempt to repair truncated JSON.
    let repaired = attempt_json_repair(&json_payload);
    serde_json::from_str::<AiDisclosurePayload>(&repaired)
        .map_err(|error| {
            let preview: String = trimmed.chars().take(200).collect();
            format!("Qwen JSON 解析失败：{}；原始内容前200字符：{}", error, preview)
        })
}

/// Strip markdown code block wrappers like ```json ... ``` or ``` ... ```
fn strip_markdown_code_block(input: &str) -> String {
    let trimmed = input.trim();

    // Handle ```json\n...\n``` and ```\n...\n```
    if trimmed.starts_with("```") {
        let without_opening = if let Some(rest) = trimmed.strip_prefix("```json") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("```JSON") {
            rest
        } else {
            trimmed.strip_prefix("```").unwrap_or(trimmed)
        };

        let body = without_opening
            .trim_start_matches(|c: char| c == '\n' || c == '\r');

        // Remove trailing ``` if present
        let body = if body.trim_end().ends_with("```") {
            let end = body.rfind("```").unwrap_or(body.len());
            &body[..end]
        } else {
            body
        };

        return body.trim().to_string();
    }

    trimmed.to_string()
}

/// Extract the outermost balanced JSON object by tracking brace depth.
/// Handles strings (skipping braces inside quotes) for correctness.
fn extract_balanced_json(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let start = input.find('{')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;

    for i in start..bytes.len() {
        let ch = bytes[i];

        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == b'\\' && in_string {
            escape_next = true;
            continue;
        }

        if ch == b'"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        if ch == b'{' {
            depth += 1;
        } else if ch == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(input[start..=i].to_string());
            }
        }
    }

    // If we never balanced, return everything from first '{' to end
    // so the repair step can try to fix it.
    if depth > 0 {
        return Some(input[start..].to_string());
    }

    None
}

/// Attempt to repair truncated JSON by appending missing closing braces.
fn attempt_json_repair(json: &str) -> String {
    let mut repaired = json.trim_end().to_string();

    // Remove trailing comma that may precede a missing brace
    if repaired.ends_with(',') {
        repaired.pop();
    }

    // Add an empty string value if the JSON ends with a key colon
    let trimmed_end = repaired.trim_end();
    if trimmed_end.ends_with(':') || trimmed_end.ends_with(": ") {
        repaired.push_str("\"\"");
    }

    // Close any unclosed strings
    let quote_count = repaired.chars().filter(|c| *c == '"').count()
        - repaired.matches("\\\"").count();
    if quote_count % 2 != 0 {
        repaired.push('"');
    }

    // Count unbalanced braces and close them
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for ch in repaired.chars() {
        if esc { esc = false; continue; }
        if ch == '\\' && in_str { esc = true; continue; }
        if ch == '"' { in_str = !in_str; continue; }
        if in_str { continue; }
        if ch == '{' { depth += 1; }
        if ch == '}' { depth -= 1; }
    }

    for _ in 0..depth {
        repaired.push('}');
    }

    repaired
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
            ("title", json_value_to_text(&self.title)),
            ("field", json_value_to_text(&self.field)),
            ("background", json_value_to_text(&self.background)),
            ("purpose", json_value_to_text(&self.purpose)),
            ("technical_solution", json_value_to_text(&self.technical_solution)),
            ("benefits", json_value_to_text(&self.benefits)),
            ("figures", json_value_to_text(&self.figures)),
            ("implementation", json_value_to_text(&self.implementation)),
            ("layout", json_value_to_text(&self.layout)),
            ("questions", json_value_to_text(&self.questions)),
        ])
    }
}

/// Convert any JSON value into a human-readable text string.
///
/// The AI model sometimes returns section content as arrays or objects instead
/// of plain strings (e.g. `"figures": [{"name":"图1","desc":"..."},...]`).
/// This function flattens any JSON structure into readable text so the preview
/// never shows raw `[object Object]`.
fn json_value_to_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter()
                .map(|item| json_value_to_text(item))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
        serde_json::Value::Object(obj) => {
            // Try to extract meaningful text from object values.
            // Common patterns: {"name": "图1", "description": "..."} or
            // {"step": "步骤1", "content": "..."}.
            obj.values()
                .map(|v| json_value_to_text(v))
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("；")
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
    }
}