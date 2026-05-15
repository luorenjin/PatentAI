目标：把当前 Tauri + React 默认 scaffold 演进为可演示的 PatentScribe V1 最小切片。首轮以 mock 优先，不接真实 Qwen，不做正式 docx 导出，不做 PDF 本地解析，但必须保留 PRD 的核心交互模型：左侧对话、右侧实时预览、可提交或跳过回答、可提前终止、PDF 模式可切换、下载入口可达。

范围约束：以 docs/PatentScribe-AI-SPEC-V5.md 为唯一产品依据。当前轮次只做 demo slice，不加入专利检索、权利要求生成、多代理人协作、移动端、本地部署。

建议拆解为以下 issue：

Issue 1: 定义 demo slice 的数据契约与状态模型
- 目标：冻结前后端第一轮都要遵循的请求、响应、状态边界，避免先写 UI 再返工。
- 涉及文件：src 侧新建类型文件；src-tauri 侧新建 serde 模型文件；必要时调整 src/App.tsx 和 src-tauri/src/lib.rs 的调用入口。
- 任务内容：
	- 定义消息模型，至少区分 system status、AI diagnosis、AI follow-up、engineer answer、error。
	- 定义 session stage，至少覆盖 idle、analyzing、awaiting_answer、updating_preview、advisory_complete、terminated。
	- 定义 disclosure 文档模型，明确 8 个固定章节、2 个专项输出、annotation 列表、版本标识。
	- 定义设置模型，至少覆盖 PDF 处理模式 direct-to-model 和 local-preprocess。
	- 定义 Tauri 命令输入输出模型，覆盖 initial analysis、submit answer、skip question、update settings、download placeholder。
- 完成标准：
	- 前端和 Rust 都使用显式类型，不依赖 ad-hoc string。
	- 状态结构足以支撑左侧消息和右侧预览由同一份 canonical disclosure state 驱动。
	- 类型中为后续 validation details 和 retry metadata 预留字段位置。
- 依赖：无。
- 验证：TypeScript 类型可通过编译，Rust serde 模型可通过 cargo check。

Issue 2: 替换 scaffold 页面为桌面双栏工作区骨架
- 目标：移除默认 greet 页面，建立符合 PRD 的主界面。
- 涉及文件：src/App.tsx、src/App.css，必要时新增 src/components 下的组件文件。
- 任务内容：
	- 把页面改为左对话、右预览的桌面优先布局，移动到窄屏时能折叠但不作为主设计目标。
	- 对话区至少包含消息时间线、输入区、发送按钮、跳过按钮、会话级动作区。
	- 预览区至少包含文档内容区、标注图例、缩放或视图控制占位、代理人视图开关、移除高亮按钮。
	- 顶部或侧边保留会话上下文信息，如当前阶段、PDF 模式、可下载状态。
- 完成标准：
	- 页面不再出现默认 logo、greet 表单和模板文案。
	- 双栏区域有稳定滚动区，长文本阅读不会依赖整页滚动。
	- 预览视觉上区分 AI 补充和 AI 修改两类 annotation，默认颜色遵循 PRD。
- 依赖：Issue 1 的状态模型。
- 验证：运行 yarn build 通过，页面结构能在浏览器和 tauri dev 中正确渲染。

Issue 3: 拆分前端组件与状态职责
- 目标：避免把所有逻辑堆回 App 根组件，为后续真实 AI 接入留出清晰边界。
- 涉及文件：src/components/ConversationTimeline.tsx、src/components/Composer.tsx、src/components/PreviewPanel.tsx、src/components/PreviewControls.tsx、src/components/AnnotationLegend.tsx，以及对应样式或局部类型文件。
- 任务内容：
	- 把消息列表渲染和气泡样式独立出去。
	- 把输入与动作逻辑独立为 composer。
	- 把预览内容与预览控件拆开，避免控件状态污染文档数据。
	- 区分 durable session state 与 transient UI state，例如 zoom、inputText、loading。
- 完成标准：
	- App 只负责组合和数据流，不直接承担所有渲染细节。
	- 组件边界与 PRD 的交互区块一一对应。
	- 预览始终消费一份 canonical disclosure state，而不是从消息中二次推导文本。
- 依赖：Issue 1、Issue 2。
- 验证：yarn build 通过，关键组件 props 类型清晰且无循环依赖。

Issue 4: 建立 mock Tauri 命令表面并移除 greet
- 目标：把默认 greet scaffold 替换为结构化业务命令，同时不在首轮引入真实 AI。
- 涉及文件：src-tauri/src/lib.rs，新增 src-tauri/src/commands.rs、src-tauri/src/models.rs、src-tauri/src/mock_session.rs 或类似模块。
- 任务内容：
	- 移除 greet 命令与前端调用。
	- 注册新的结构化命令，如 start_demo_session、submit_demo_answer、skip_demo_question、update_demo_settings、download_demo_snapshot。
	- 所有命令返回 serde 结构化 payload，包含 stage、messages、disclosure、settings、actions 等字段。
	- 在 Rust 普通模块中组织 mock 数据和状态演进，不把业务堆在 lib.rs。
- 完成标准：
	- tauri::generate_handler! 中只注册业务命令。
	- 命令返回值能直接驱动前端，不需要前端再解析大段字符串。
	- 返回模型中预留 validation 与 retry 的扩展位。
- 依赖：Issue 1。
- 验证：cargo check 通过；前端能成功 invoke 新命令并拿到结构化数据。

Issue 5: 打通初次分析链路与 demo 会话加载
- 目标：让应用启动后可以进入一次完整的 mock 分析开场，而不是静态页面。
- 涉及文件：前端根组件与会话初始化逻辑，Rust mock session 模块。
- 任务内容：
	- 页面加载时或用户点击“开始分析”时调用 start_demo_session。
	- 左侧生成 system status 与 AI diagnosis 初始消息。
	- 右侧加载第一版 disclosure 预览与 annotation。
	- 显示当前阶段、下一步动作和 PDF 模式状态。
- 完成标准：
	- 应用不是空壳，首次进入即可演示“已分析初稿”的产品状态。
	- 左右两栏由同一份返回 payload 同步更新。
- 依赖：Issue 2、Issue 3、Issue 4。
- 验证：yarn tauri dev 下手动刷新应用，能稳定进入 demo 会话。

Issue 6: 实现回答周期与跳过逻辑
- 目标：落地 PRD 的核心回答循环，而不是只停在首屏展示。
- 涉及文件：前端 composer、会话状态管理、Rust mock answer flow。
- 任务内容：
	- 实现 submit answer：发送回答后进入 pending，再更新消息流与预览。
	- 实现 skip question：用户跳过时也触发同样的状态迁移，但附带不同的系统说明。
	- 每次提交或跳过后，返回下一条 AI follow-up 或 advisory complete 提示。
	- 明确按钮可用性规则，避免 analyzing 和 updating_preview 阶段重复提交。
- 完成标准：
	- 回答或跳过都能触发预览更新。
	- 状态迁移清楚，不依赖多个松散布尔值。
	- AI 判断信息已充足只作为建议，不阻止用户继续操作或终止。
- 依赖：Issue 1、Issue 3、Issue 4、Issue 5。
- 验证：手动完成至少一轮 submit 和一轮 skip，确认消息与预览同步变化。

Issue 7: 落地预览标注、代理人视图与移除高亮
- 目标：让右侧预览不仅显示文本，还体现 AI 补充和修改的可视差异。
- 涉及文件：PreviewPanel、AnnotationLegend、PreviewControls、前端状态文件。
- 任务内容：
	- 为 AI added 和 AI rewritten 建立不同 annotation 渲染样式。
	- 实现代理人视图开关，至少支持“聚焦高亮内容”的 demo 行为。
	- 实现移除所有高亮的 demo 行为，允许用户查看清洁版文本。
	- 保证视图切换不修改底层 canonical disclosure 数据，只改变展示层。
- 完成标准：
	- 蓝色和黄色标注可区分且图例一致。
	- 代理人视图与清除高亮都能即时生效。
	- 标注控制不会破坏后续回答周期。
- 依赖：Issue 2、Issue 3、Issue 5、Issue 6。
- 验证：手动切换两种视图控制，确认预览展示变化符合预期。

Issue 8: 加入提前终止和下载入口占位
- 目标：满足 PRD 中“工程师可随时终止并下载当前版本”的可达性要求。
- 涉及文件：会话动作区、Rust download placeholder 命令、前端交互逻辑。
- 任务内容：
	- 在界面持续显示“下载当前版本”和“提前终止”动作。
	- 提前终止后将 session stage 切换为 terminated，但仍允许查看当前预览。
	- 下载行为首轮可返回占位结果，如提示“导出能力待接入”，但链路必须可调用。
	- 若合适，加入“是否包含 AI 追问回答记录”的占位选项。
- 完成标准：
	- 用户无需等待 AI 建议 complete 才能终止或下载。
	- 终止行为不会让页面进入不可恢复的错误状态。
	- 下载入口不是纯静态按钮，而是有真实命令链路或明确占位反馈。
- 依赖：Issue 4、Issue 5。
- 验证：手动执行 terminate 和 download placeholder，确认状态与反馈正确。

Issue 9: 实现 PDF 模式设置与最小持久化
- 目标：把 PRD 的 PDF 双模式作为真实设置保留下来，即使首轮不实现本地解析。
- 涉及文件：前端设置控件、Rust settings 命令、必要的本地持久化位置。
- 任务内容：
	- 提供 direct-to-model 和 local-preprocess 两个显式选项，并标注默认推荐项。
	- 设置变更立即生效，并在 UI 中可见当前值。
	- 选择一个最小持久化方案保存设置，例如 Tauri store、配置文件或本地状态占位。
	- 后端显式接收并返回该模式，避免仅存在于前端。
- 完成标准：
	- 刷新或重开应用后设置可恢复，至少在 demo 范围内表现为持久化。
	- 前后端对 PDF 模式的枚举值一致。
- 依赖：Issue 1、Issue 4。
- 验证：切换设置后重启应用，确认模式仍然保持。

Issue 10: 为后续真实能力预留 backend 模块边界
- 目标：当前虽是 mock，但结构上已经能承接真实 ingest、analyze、validate、export。
- 涉及文件：src-tauri/src 下的模块组织与 public types。
- 任务内容：
	- 抽出 ingest、analyze、validate、export、session 或等价模块边界。
	- 明确 mock 逻辑所在位置，不与未来真实实现混杂。
	- 在类型中预留 validation failure、retry exhaustion、unsupported format、recoverable error 等结果分支。
	- 为 8 章节 + 2 专项输出校验结果设计机器可读结构，但本轮不实现校验逻辑。
- 完成标准：
	- lib.rs 只做 command 注册和薄入口调度。
	- Rust 模块命名与 PRD pipeline 对齐，后续接入真实 AI 不需要再大改目录结构。
- 依赖：Issue 4。
- 验证：cargo check 通过，模块依赖方向清晰。

Issue 11: 做最窄验证与演示脚本整理
- 目标：把这轮工作收敛到可演示、可复验，而不是“代码看起来差不多”。
- 涉及文件：必要时更新 README 或新建极短开发说明；主要是运行验证命令。
- 任务内容：
	- 跑 yarn build 验证前端。
	- 跑 cargo check 验证 Rust。
	- 跑 yarn tauri dev，手动走一遍 demo：加载会话、收到诊断、回答、跳过、预览更新、切换 PDF 模式、提前终止、下载占位。
	- 整理一份最短演示脚本或验收清单，便于下一轮继续开发或评审。
- 完成标准：
	- 至少有一条可重复的 end-to-end demo 路径。
	- 若存在未实现能力，必须在演示脚本中明确标记为 placeholder，而不是隐含缺失。
- 依赖：Issue 2 至 Issue 10 完成到可运行状态。
- 验证：上述三类验证都执行完毕，并记录结果。

建议执行顺序：
1. 先做 Issue 1。
2. 然后并行推进 Issue 2 和 Issue 4。
3. 在 UI 骨架和命令表面稳定后完成 Issue 3。
4. 再串联 Issue 5、Issue 6、Issue 7、Issue 8。
5. Issue 9 和 Issue 10 可穿插在命令表面完成之后。
6. 最后做 Issue 11。

建议第一批提交范围：
- Issue 1
- Issue 2
- Issue 4
- Issue 5
- Issue 6

这批完成后，产品已经从默认 scaffold 进入“可演示主链路可跑通”的状态。之后再补 Issue 7、Issue 8、Issue 9、Issue 10、Issue 11，把它收敛成可评审、可继续扩展的 demo 基线。
