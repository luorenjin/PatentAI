use crate::models::{
    Annotation, AnnotationKind, ChatMessage, DemoSettings, DisclosureDocument, DisclosureSection,
    DownloadRequest, DownloadResponse, DownloadStatus, MessageIntent, MessageRole,
    PdfProcessingMode, RetryMetadata, SessionActions, SessionProgress, SessionRequest,
    SessionSnapshot, SessionStage, StartDemoSessionRequest, UpdateSettingsRequest,
};
use crate::validation::validate_disclosure_document;

const QUESTION_TOTAL: usize = 2;

pub fn start_session(request: StartDemoSessionRequest) -> SessionSnapshot {
    let source_draft = request.source_draft.trim().to_string();
    let settings = DemoSettings {
        pdf_mode: request.settings.pdf_mode,
    };

    refresh_validation(SessionSnapshot {
        session_id: "demo-session-001".to_string(),
        stage: SessionStage::AwaitingAnswer,
        source_draft,
        messages: vec![
            message(
                "m1",
                MessageRole::System,
                MessageIntent::Status,
                "已加载当前输入的交底书草稿，正在以 PatentScribe 工作流展示第一轮诊断。",
            ),
            message(
                "m2",
                MessageRole::Assistant,
                MessageIntent::Diagnosis,
                "诊断报告：当前草稿已具备基础技术背景，但对现有技术对比不足，新颖性论证偏弱；具体实施方式缺少参数区间与控制条件。",
            ),
            message(
                "m3",
                MessageRole::Assistant,
                MessageIntent::FollowUp,
                question_for_index(0),
            ),
        ],
        disclosure: disclosure_for_index(0),
        settings,
        actions: active_actions(),
        progress: SessionProgress {
            current_question_index: 0,
            total_questions: QUESTION_TOTAL,
        },
        validation_issues: Vec::new(),
        retry_metadata: RetryMetadata {
            attempt_count: 0,
            max_attempts: 3,
        },
    })
}

pub fn submit_answer(mut session: SessionSnapshot, answer: String) -> SessionSnapshot {
    let current_index = session.progress.current_question_index;
    session.messages.push(message(
        &format!("answer-{}", current_index + 1),
        MessageRole::Engineer,
        MessageIntent::Answer,
        &answer,
    ));
    session.messages.push(message(
        &format!("status-{}", current_index + 1),
        MessageRole::System,
        MessageIntent::Status,
        "已根据你的补充更新右侧交底书预览。",
    ));

    advance_session(session, false)
}

pub fn skip_question(mut session: SessionSnapshot) -> SessionSnapshot {
    let current_index = session.progress.current_question_index;
    session.messages.push(message(
        &format!("skip-{}", current_index + 1),
        MessageRole::System,
        MessageIntent::Status,
        "已跳过当前追问，系统将继续保留待补充标记并进入下一项引导。",
    ));

    advance_session(session, true)
}

pub fn update_settings(mut request: UpdateSettingsRequest) -> SessionSnapshot {
    request.session.settings = request.settings;
    request.session.messages.push(message(
        "settings-updated",
        MessageRole::System,
        MessageIntent::Status,
        &format!(
            "PDF 处理模式已更新为{}。",
            match request.session.settings.pdf_mode {
                PdfProcessingMode::DirectToModel => "直接提交大模型",
                PdfProcessingMode::LocalPreprocess => "本地预处理",
            }
        ),
    ));

    refresh_validation(request.session)
}

pub fn terminate_session(mut request: SessionRequest) -> SessionSnapshot {
    request.session.stage = SessionStage::Terminated;
    request.session.actions.can_submit = false;
    request.session.actions.can_skip = false;
    request.session.actions.can_terminate = false;
    request.session.messages.push(message(
        "terminated",
        MessageRole::System,
        MessageIntent::Advisory,
        "当前会话已提前终止。你仍可查看右侧预览并下载当前版本。",
    ));

    refresh_validation(request.session)
}

pub fn download_snapshot(request: DownloadRequest) -> DownloadResponse {
    let validation_issue_count = validate_disclosure_document(&request.session.disclosure).len();

    DownloadResponse {
        status: DownloadStatus::Placeholder,
        file_name: format!(
            "patentscribe-demo-v{}.docx",
            request.session.disclosure.version
        ),
        message: format!(
            "导出命令链路已接通；{}。{}",
            if request.include_answer_history {
                "当前占位导出将包含 AI 追问记录"
            } else {
                "当前占位导出默认不包含 AI 追问记录"
            },
            if validation_issue_count == 0 {
                "当前版本已通过 demo 级输出格式校验"
            } else {
                "当前版本仍存在输出格式提示，正式导出前建议继续修正"
            }
        ),
        included_answer_history: request.include_answer_history,
        validation_issue_count,
    }
}

fn advance_session(mut session: SessionSnapshot, skipped: bool) -> SessionSnapshot {
    let next_index = session.progress.current_question_index + 1;
    session.disclosure = disclosure_for_index(next_index);

    if next_index >= QUESTION_TOTAL {
        session.progress.current_question_index = QUESTION_TOTAL;
        session.stage = SessionStage::AdvisoryComplete;
        session.actions = advisory_actions();
        session.messages.push(message(
            "advisory-complete",
            MessageRole::Assistant,
            MessageIntent::Advisory,
            if skipped {
                "当前交底书已达到演示级完整度。仍有待工程师补充项，但你现在可以终止或下载当前版本。"
            } else {
                "当前交底书信息已较为完整，可以继续补充，也可以直接下载当前版本。"
            },
        ));
    } else {
        session.progress.current_question_index = next_index;
        session.stage = SessionStage::AwaitingAnswer;
        session.actions = active_actions();
        session.messages.push(message(
            &format!("follow-up-{}", next_index + 1),
            MessageRole::Assistant,
            MessageIntent::FollowUp,
            question_for_index(next_index),
        ));
    }

    refresh_validation(session)
}

fn refresh_validation(mut session: SessionSnapshot) -> SessionSnapshot {
    session.validation_issues = validate_disclosure_document(&session.disclosure);
    session
}

fn message(id: &str, role: MessageRole, intent: MessageIntent, content: &str) -> ChatMessage {
    ChatMessage {
        id: id.to_string(),
        role,
        intent,
        content: content.to_string(),
        timestamp: "2026-05-15T09:00:00+08:00".to_string(),
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

fn question_for_index(index: usize) -> &'static str {
    match index {
        0 => "为了强化新颖性论证，请补充：现有技术通常采用什么结构或流程？你的方案与它的关键差异是什么？",
        1 => "为了让具体实施方式可落地，请补充：关键参数范围、触发条件或控制逻辑分别是什么？",
        _ => "当前没有新的追问。",
    }
}

fn disclosure_for_index(index: usize) -> DisclosureDocument {
    let technical_solution = match index {
        0 => "本方案通过多级调度模块协调边缘节点与云端分析链路，以降低交底信息整理过程中的往返耗时。【待工程师补充】现有技术的典型链路与本方案的差异点。",
        1 => "本方案通过多级调度模块协调边缘节点与云端分析链路，并在任务切换时引入上下文快照与优先级阈值控制，使交底信息能按创新点、现有技术对比和实施参数三个维度渐进完善。",
        _ => "本方案通过多级调度模块协调边缘节点与云端分析链路，并在任务切换时引入上下文快照、优先级阈值控制和参数回填机制，以支持交底草稿在多轮补充中持续收敛。建议后续补充实验参数与异常处理策略。",
    };

    let implementation = match index {
        0 => "实施例一：【待工程师补充】关键阈值、切换条件和输入输出关系。实施例二：【待工程师补充】异常场景下的处理策略。",
        1 => "实施例一：当输入任务优先级高于阈值时，系统保留上一轮快照并优先写回新颖性对比信息。实施例二：【待工程师补充】异常场景下的恢复策略与默认参数范围。",
        _ => "实施例一：当输入任务优先级高于阈值时，系统保留上一轮快照并优先写回新颖性对比信息。实施例二：当检测到参数缺失时，系统以待补充标记保留空位，并提示工程师补充控制条件与范围值。",
    };

    let annotations = match index {
        0 => vec![
            Annotation {
                id: "a1".to_string(),
                kind: AnnotationKind::Addition,
                section_id: "purpose".to_string(),
                excerpt: "降低交底信息整理过程中的往返耗时".to_string(),
                note: "基于系统诊断补齐发明目的。".to_string(),
            },
            Annotation {
                id: "a2".to_string(),
                kind: AnnotationKind::Rewrite,
                section_id: "technical_solution".to_string(),
                excerpt: "多级调度模块协调边缘节点与云端分析链路".to_string(),
                note: "将原始口语化描述改写为可写入交底书的工程表述。".to_string(),
            },
        ],
        1 => vec![
            Annotation {
                id: "a3".to_string(),
                kind: AnnotationKind::Addition,
                section_id: "technical_solution".to_string(),
                excerpt: "引入上下文快照与优先级阈值控制".to_string(),
                note: "基于工程师回答补充控制逻辑。".to_string(),
            },
            Annotation {
                id: "a4".to_string(),
                kind: AnnotationKind::Rewrite,
                section_id: "implementation".to_string(),
                excerpt: "系统保留上一轮快照并优先写回新颖性对比信息".to_string(),
                note: "将实施步骤重写为更清晰的执行顺序。".to_string(),
            },
        ],
        _ => vec![
            Annotation {
                id: "a5".to_string(),
                kind: AnnotationKind::Addition,
                section_id: "implementation".to_string(),
                excerpt: "以待补充标记保留空位".to_string(),
                note: "保留缺失信息提示，避免虚构核心技术事实。".to_string(),
            },
            Annotation {
                id: "a6".to_string(),
                kind: AnnotationKind::Rewrite,
                section_id: "benefits".to_string(),
                excerpt: "减少工程师与代理人之间的来回沟通".to_string(),
                note: "将效果描述改写为可核验的业务收益。".to_string(),
            },
        ],
    };

    DisclosureDocument {
        version: (index as u32) + 1,
        sections: vec![
            section("title", "一、发明名称", "一种用于专利交底信息渐进完善的协同生成方法"),
            section("field", "二、技术领域", "本发明涉及 AI 辅助专利交底书生成与技术信息整理领域，尤其涉及一种对话式渐进补全方法。"),
            section("background", "三、背景技术及现有缺陷", "现有交底书撰写流程通常依赖工程师先提交不完整初稿，再由代理人多轮追问补足。该流程存在技术对比不充分、实施细节遗漏、沟通周期长等问题。"),
            section("purpose", "四、发明目的", "本方案旨在减少交底书从初稿到可交付版本之间的往返轮次，并保持待补充信息可见。"),
            section("technical_solution", "五、技术方案", technical_solution),
            section("benefits", "六、有益效果", "| 维度 | 技术突破点 | 效果与价值体现 |\n| --- | --- | --- |\n| 沟通效率 | 通过会话驱动的渐进补全 | 缩短工程师与代理人之间的来回沟通 |\n| 信息完整度 | 以待补充标记保留缺口 | 降低遗漏关键实施细节的风险 |"),
            section("figures", "七、附图说明", "图 1 为对话驱动的交底完善流程示意图；图 2 为多级调度模块的结构示意图。"),
            section("implementation", "八、具体实施方式", implementation),
            section("layout", "🔍 专利挖掘与布局建议", "| 布局方向 | 保护策略 | 建议权利要求架构 |\n| --- | --- | --- |\n| 会话驱动补全 | 覆盖追问触发逻辑与状态迁移 | 独立方法权利要求 + 系统权利要求 |\n| 参数回填机制 | 覆盖待补充标记与参数写回策略 | 方法从属权利要求 |"),
            section(
                "questions",
                "🛠️ 需工程师补齐的关键改进建议",
                match index {
                    0 => "请补充异常场景下的恢复策略，并说明关键阈值的来源与范围。",
                    _ => "Q1：请补充异常场景下的恢复策略，以支撑可实施性判断。\nQ2：请补充关键阈值的来源和范围，以强化技术方案的落地边界。",
                },
            ),
        ],
        annotations,
    }
}

fn section(id: &str, title: &str, content: &str) -> DisclosureSection {
    DisclosureSection {
        id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
    }
}