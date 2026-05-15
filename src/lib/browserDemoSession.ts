import type {
  Annotation,
  ChatMessage,
  DisclosureDocument,
  DisclosureSection,
  DownloadRequest,
  DownloadResponse,
  SessionActions,
  SessionRequest,
  SessionSnapshot,
  StartDemoSessionRequest,
  UpdateSettingsRequest,
  ValidationIssue,
} from "../types/patentSession";

const QUESTION_TOTAL = 2;
const STATIC_TIMESTAMP = "2026-05-15T09:00:00+08:00";
const REQUIRED_SECTIONS = [
  ["title", "一、发明名称"],
  ["field", "二、技术领域"],
  ["background", "三、背景技术及现有缺陷"],
  ["purpose", "四、发明目的"],
  ["technical_solution", "五、技术方案"],
  ["benefits", "六、有益效果"],
  ["figures", "七、附图说明"],
  ["implementation", "八、具体实施方式"],
  ["layout", "🔍 专利挖掘与布局建议"],
  ["questions", "🛠️ 需工程师补齐的关键改进建议"],
] as const;

const BENEFITS_TABLE_HEADER = "| 维度 | 技术突破点 | 效果与价值体现 |";
const LAYOUT_TABLE_HEADER = "| 布局方向 | 保护策略 | 建议权利要求架构 |";

export function startBrowserDemoSession(
  request: StartDemoSessionRequest,
): Promise<SessionSnapshot> {
  const sourceDraft = request.sourceDraft.trim();

  if (!sourceDraft) {
    return Promise.reject(new Error("请先输入待分析的交底草稿，再启动会话。"));
  }

  return Promise.resolve(
    refreshValidation({
      sessionId: "demo-session-001",
      stage: "awaiting_answer",
      sourceDraft,
      messages: [
        message(
          "m1",
          "system",
          "status",
          "已加载当前输入的交底书草稿，正在以 PatentScribe 工作流展示第一轮诊断。",
        ),
        message(
          "m2",
          "assistant",
          "diagnosis",
          "诊断报告：当前草稿已具备基础技术背景，但对现有技术对比不足，新颖性论证偏弱；具体实施方式缺少参数区间与控制条件。",
        ),
        message("m3", "assistant", "follow_up", questionForIndex(0)),
      ],
      disclosure: disclosureForIndex(0),
      settings: {
        pdfMode: request.settings.pdfMode,
      },
      actions: activeActions(),
      progress: {
        currentQuestionIndex: 0,
        totalQuestions: QUESTION_TOTAL,
      },
      validationIssues: [],
      retryMetadata: {
        attemptCount: 0,
        maxAttempts: 3,
      },
    }),
  );
}

export function submitBrowserDemoAnswer(
  session: SessionSnapshot,
  answer: string,
): Promise<SessionSnapshot> {
  const currentIndex = session.progress.currentQuestionIndex;
  const nextSession: SessionSnapshot = {
    ...session,
    messages: [
      ...session.messages,
      message(
        `answer-${currentIndex + 1}`,
        "engineer",
        "answer",
        answer,
      ),
      message(
        `status-${currentIndex + 1}`,
        "system",
        "status",
        "已根据你的补充更新右侧交底书预览。",
      ),
    ],
  };

  return Promise.resolve(advanceSession(nextSession, false));
}

export function skipBrowserDemoQuestion(
  session: SessionSnapshot,
): Promise<SessionSnapshot> {
  const currentIndex = session.progress.currentQuestionIndex;
  const nextSession: SessionSnapshot = {
    ...session,
    messages: [
      ...session.messages,
      message(
        `skip-${currentIndex + 1}`,
        "system",
        "status",
        "已跳过当前追问，系统将继续保留待补充标记并进入下一项引导。",
      ),
    ],
  };

  return Promise.resolve(advanceSession(nextSession, true));
}

export function updateBrowserDemoSettings(
  request: UpdateSettingsRequest,
): Promise<SessionSnapshot> {
  const nextSession = refreshValidation({
    ...request.session,
    settings: request.settings,
    messages: [
      ...request.session.messages,
      message(
        "settings-updated",
        "system",
        "status",
        `PDF 处理模式已更新为${
          request.settings.pdfMode === "direct_to_model"
            ? "直接提交大模型"
            : "本地预处理"
        }。`,
      ),
    ],
  });

  return Promise.resolve(nextSession);
}

export function terminateBrowserDemoSession(
  request: SessionRequest,
): Promise<SessionSnapshot> {
  return Promise.resolve(
    refreshValidation({
      ...request.session,
      stage: "terminated",
      actions: {
        ...request.session.actions,
        canSubmit: false,
        canSkip: false,
        canTerminate: false,
      },
      messages: [
        ...request.session.messages,
        message(
          "terminated",
          "system",
          "advisory",
          "当前会话已提前终止。你仍可查看右侧预览并下载当前版本。",
        ),
      ],
    }),
  );
}

export function downloadBrowserDemoSnapshot(
  request: DownloadRequest,
): Promise<DownloadResponse> {
  const validationIssueCount = validateDisclosureDocument(
    request.session.disclosure,
  ).length;

  return Promise.resolve({
    status: "placeholder",
    fileName: `patentscribe-demo-v${request.session.disclosure.version}.docx`,
    message: `导出命令链路已接通；${
      request.includeAnswerHistory
        ? "当前占位导出将包含 AI 追问记录"
        : "当前占位导出默认不包含 AI 追问记录"
    }。${
      validationIssueCount === 0
        ? "当前版本已通过 demo 级输出格式校验"
        : "当前版本仍存在输出格式提示，正式导出前建议继续修正"
    }`,
    includedAnswerHistory: request.includeAnswerHistory,
    validationIssueCount,
  });
}

function advanceSession(
  session: SessionSnapshot,
  skipped: boolean,
): SessionSnapshot {
  const nextIndex = session.progress.currentQuestionIndex + 1;
  const nextSession: SessionSnapshot = {
    ...session,
    disclosure: disclosureForIndex(nextIndex),
  };

  if (nextIndex >= QUESTION_TOTAL) {
    nextSession.progress = {
      currentQuestionIndex: QUESTION_TOTAL,
      totalQuestions: session.progress.totalQuestions,
    };
    nextSession.stage = "advisory_complete";
    nextSession.actions = advisoryActions();
    nextSession.messages = [
      ...nextSession.messages,
      message(
        "advisory-complete",
        "assistant",
        "advisory",
        skipped
          ? "当前交底书已达到演示级完整度。仍有待工程师补充项，但你现在可以终止或下载当前版本。"
          : "当前交底书信息已较为完整，可以继续补充，也可以直接下载当前版本。",
      ),
    ];
  } else {
    nextSession.progress = {
      currentQuestionIndex: nextIndex,
      totalQuestions: session.progress.totalQuestions,
    };
    nextSession.stage = "awaiting_answer";
    nextSession.actions = activeActions();
    nextSession.messages = [
      ...nextSession.messages,
      message(
        `follow-up-${nextIndex + 1}`,
        "assistant",
        "follow_up",
        questionForIndex(nextIndex),
      ),
    ];
  }

  return refreshValidation(nextSession);
}

function refreshValidation(session: SessionSnapshot): SessionSnapshot {
  return {
    ...session,
    validationIssues: validateDisclosureDocument(session.disclosure),
  };
}

function message(
  id: string,
  role: ChatMessage["role"],
  intent: ChatMessage["intent"],
  content: string,
): ChatMessage {
  return {
    id,
    role,
    intent,
    content,
    timestamp: STATIC_TIMESTAMP,
  };
}

function activeActions(): SessionActions {
  return {
    canSubmit: true,
    canSkip: true,
    canDownload: true,
    canTerminate: true,
  };
}

function advisoryActions(): SessionActions {
  return {
    canSubmit: true,
    canSkip: false,
    canDownload: true,
    canTerminate: true,
  };
}

function questionForIndex(index: number): string {
  switch (index) {
    case 0:
      return "为了强化新颖性论证，请补充：现有技术通常采用什么结构或流程？你的方案与它的关键差异是什么？";
    case 1:
      return "为了让具体实施方式可落地，请补充：关键参数范围、触发条件或控制逻辑分别是什么？";
    default:
      return "当前没有新的追问。";
  }
}

function disclosureForIndex(index: number): DisclosureDocument {
  const technicalSolution =
    index === 0
      ? "本方案通过多级调度模块协调边缘节点与云端分析链路，以降低交底信息整理过程中的往返耗时。【待工程师补充】现有技术的典型链路与本方案的差异点。"
      : index === 1
        ? "本方案通过多级调度模块协调边缘节点与云端分析链路，并在任务切换时引入上下文快照与优先级阈值控制，使交底信息能按创新点、现有技术对比和实施参数三个维度渐进完善。"
        : "本方案通过多级调度模块协调边缘节点与云端分析链路，并在任务切换时引入上下文快照、优先级阈值控制和参数回填机制，以支持交底草稿在多轮补充中持续收敛。建议后续补充实验参数与异常处理策略。";

  const implementation =
    index === 0
      ? "实施例一：【待工程师补充】关键阈值、切换条件和输入输出关系。实施例二：【待工程师补充】异常场景下的处理策略。"
      : index === 1
        ? "实施例一：当输入任务优先级高于阈值时，系统保留上一轮快照并优先写回新颖性对比信息。实施例二：【待工程师补充】异常场景下的恢复策略与默认参数范围。"
        : "实施例一：当输入任务优先级高于阈值时，系统保留上一轮快照并优先写回新颖性对比信息。实施例二：当检测到参数缺失时，系统以待补充标记保留空位，并提示工程师补充控制条件与范围值。";

  const annotations: Annotation[] =
    index === 0
      ? [
          {
            id: "a1",
            kind: "addition",
            sectionId: "purpose",
            excerpt: "降低交底信息整理过程中的往返耗时",
            note: "基于系统诊断补齐发明目的。",
          },
          {
            id: "a2",
            kind: "rewrite",
            sectionId: "technical_solution",
            excerpt: "多级调度模块协调边缘节点与云端分析链路",
            note: "将原始口语化描述改写为可写入交底书的工程表述。",
          },
        ]
      : index === 1
        ? [
            {
              id: "a3",
              kind: "addition",
              sectionId: "technical_solution",
              excerpt: "引入上下文快照与优先级阈值控制",
              note: "基于工程师回答补充控制逻辑。",
            },
            {
              id: "a4",
              kind: "rewrite",
              sectionId: "implementation",
              excerpt: "系统保留上一轮快照并优先写回新颖性对比信息",
              note: "将实施步骤重写为更清晰的执行顺序。",
            },
          ]
        : [
            {
              id: "a5",
              kind: "addition",
              sectionId: "implementation",
              excerpt: "以待补充标记保留空位",
              note: "保留缺失信息提示，避免虚构核心技术事实。",
            },
            {
              id: "a6",
              kind: "rewrite",
              sectionId: "benefits",
              excerpt: "减少工程师与代理人之间的来回沟通",
              note: "将效果描述改写为可核验的业务收益。",
            },
          ];

  return {
    version: index + 1,
    sections: [
      section("title", "一、发明名称", "一种用于专利交底信息渐进完善的协同生成方法"),
      section(
        "field",
        "二、技术领域",
        "本发明涉及 AI 辅助专利交底书生成与技术信息整理领域，尤其涉及一种对话式渐进补全方法。",
      ),
      section(
        "background",
        "三、背景技术及现有缺陷",
        "现有交底书撰写流程通常依赖工程师先提交不完整初稿，再由代理人多轮追问补足。该流程存在技术对比不充分、实施细节遗漏、沟通周期长等问题。",
      ),
      section(
        "purpose",
        "四、发明目的",
        "本方案旨在减少交底书从初稿到可交付版本之间的往返轮次，并保持待补充信息可见。",
      ),
      section("technical_solution", "五、技术方案", technicalSolution),
      section(
        "benefits",
        "六、有益效果",
        "| 维度 | 技术突破点 | 效果与价值体现 |\n| --- | --- | --- |\n| 沟通效率 | 通过会话驱动的渐进补全 | 缩短工程师与代理人之间的来回沟通 |\n| 信息完整度 | 以待补充标记保留缺口 | 降低遗漏关键实施细节的风险 |",
      ),
      section(
        "figures",
        "七、附图说明",
        "图 1 为对话驱动的交底完善流程示意图；图 2 为多级调度模块的结构示意图。",
      ),
      section("implementation", "八、具体实施方式", implementation),
      section(
        "layout",
        "🔍 专利挖掘与布局建议",
        "| 布局方向 | 保护策略 | 建议权利要求架构 |\n| --- | --- | --- |\n| 会话驱动补全 | 覆盖追问触发逻辑与状态迁移 | 独立方法权利要求 + 系统权利要求 |\n| 参数回填机制 | 覆盖待补充标记与参数写回策略 | 方法从属权利要求 |",
      ),
      section(
        "questions",
        "🛠️ 需工程师补齐的关键改进建议",
        index === 0
          ? "请补充异常场景下的恢复策略，并说明关键阈值的来源与范围。"
          : "Q1：请补充异常场景下的恢复策略，以支撑可实施性判断。\nQ2：请补充关键阈值的来源和范围，以强化技术方案的落地边界。",
      ),
    ],
    annotations,
  };
}

function section(id: string, title: string, content: string): DisclosureSection {
  return {
    id,
    title,
    content,
  };
}

function validateDisclosureDocument(
  document: DisclosureDocument,
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  validateRequiredSections(document, issues);
  validateTableHeader(
    document,
    "benefits",
    BENEFITS_TABLE_HEADER,
    "benefits_table_format",
    "第六部分“有益效果”必须使用“维度 | 技术突破点 | 效果与价值体现”表格格式。",
    issues,
  );
  validateTableHeader(
    document,
    "layout",
    LAYOUT_TABLE_HEADER,
    "layout_table_format",
    "“专利挖掘与布局建议”必须使用“布局方向 | 保护策略 | 建议权利要求架构”表格格式。",
    issues,
  );
  validateQuestionsFormat(document, issues);
  validateMissingFactMarkers(document, issues);

  return issues;
}

function validateRequiredSections(
  document: DisclosureDocument,
  issues: ValidationIssue[],
) {
  if (document.sections.length !== REQUIRED_SECTIONS.length) {
    issues.push({
      code: "section_count_mismatch",
      message: `当前输出共包含 ${document.sections.length} 个章节/专项，预期为 ${REQUIRED_SECTIONS.length} 个。`,
      level: "error",
    });
  }

  for (const [index, [expectedId, expectedTitle]] of REQUIRED_SECTIONS.entries()) {
    const currentSection = document.sections[index];

    if (!currentSection) {
      issues.push({
        code: `missing_section_${expectedId}`,
        message: `缺少必需章节/专项“${expectedTitle}”。`,
        sectionId: expectedId,
        level: "error",
      });
      continue;
    }

    if (
      currentSection.id !== expectedId ||
      currentSection.title !== expectedTitle
    ) {
      issues.push({
        code: `section_order_${index}`,
        message: `第 ${index + 1} 个章节/专项应为“${expectedTitle}”，当前实际为“${currentSection.title}”。`,
        sectionId: currentSection.id,
        level: "error",
      });
      continue;
    }

    if (!currentSection.content.trim()) {
      issues.push({
        code: `empty_section_${expectedId}`,
        message: `${expectedTitle} 当前内容为空。`,
        sectionId: expectedId,
        level: "error",
      });
    }
  }
}

function validateTableHeader(
  document: DisclosureDocument,
  sectionId: string,
  expectedHeader: string,
  code: string,
  validationMessage: string,
  issues: ValidationIssue[],
) {
  const currentSection = findSection(document, sectionId);
  if (!currentSection || currentSection.content.includes(expectedHeader)) {
    return;
  }

  issues.push({
    code,
    message: validationMessage,
    sectionId,
    level: "error",
  });
}

function validateQuestionsFormat(
  document: DisclosureDocument,
  issues: ValidationIssue[],
) {
  const currentSection = findSection(document, "questions");
  if (!currentSection) {
    return;
  }

  const hasQuestionFormat = currentSection.content.split("\n").some((line) => {
    const trimmed = line.trim();
    return /^Q\d/.test(trimmed);
  });

  if (hasQuestionFormat) {
    return;
  }

  issues.push({
    code: "questions_q_format",
    message: "“需工程师补齐的关键改进建议”必须使用 Q1/Q2 问答式格式。",
    sectionId: "questions",
    level: "error",
  });
}

function validateMissingFactMarkers(
  document: DisclosureDocument,
  issues: ValidationIssue[],
) {
  for (const sectionId of ["technical_solution", "implementation"]) {
    const currentSection = findSection(document, sectionId);
    if (
      !currentSection ||
      !currentSection.content.includes("待补充") ||
      currentSection.content.includes("【待工程师补充】")
    ) {
      continue;
    }

    issues.push({
      code: `missing_marker_${sectionId}`,
      message: `${currentSection.title} 中出现缺失信息提示时，应使用“【待工程师补充】”统一标记。`,
      sectionId,
      level: "warning",
    });
  }
}

function findSection(
  document: DisclosureDocument,
  sectionId: string,
): DisclosureSection | undefined {
  return document.sections.find((sectionItem) => sectionItem.id === sectionId);
}