---
description: "Use when breaking the PatentScribe PRD into milestones, user stories, or executable implementation tasks for planning and sequencing work."
name: "PatentScribe PRD Feature Breakdown"
argument-hint: "可选：输入 V1、某个 Story、某个里程碑或关注能力，如 Story 1 / 输出格式校验 / PDF 处理"
agent: "plan"
---
请基于 [docs/PatentScribe-AI-SPEC-V5.md](docs/PatentScribe-AI-SPEC-V5.md)、[AGENTS.md](AGENTS.md) 和当前仓库状态，把 PatentScribe AI 的 PRD 拆成后续可执行的开发任务。

如果我提供了参数，请优先聚焦该范围；如果没有提供参数，默认按 V1 范围进行拆解，并结合当前仓库仍接近默认 Tauri 模板的现实状态安排实施顺序。

工作要求：
- 先识别当前实现现状与 PRD 之间的差距，但不要开始写代码。
- 严格遵守现有项目约束：Tauri 2 + React + TypeScript 前端，Rust/Tauri 后端，Yarn 工作流，产品范围以 PRD 为准。
- 不要把非目标范围混进 V1，例如专利检索、权利要求书生成、移动端、本地部署、多代理人协作。
- 任务必须可执行，尽量拆到单次开发会话可推进的粒度；能推断到文件或目录时，请指出落点。
- 对不明确或 PRD 尚未定案的地方，标记为开放问题或前置依赖，不要自行脑补实现细节。

按以下格式输出：

## 范围判断
- 本次聚焦：
- 适用的 PRD 范围：
- 明确排除项：

## 里程碑顺序
输出一个表格，列为：里程碑 | 用户结果 | 主要实现面 | 前置依赖 | 完成标志

## 可执行任务清单
输出一个表格，列为：任务 ID | 所属里程碑 | 层级（Frontend/Rust/Shared） | 具体改动 | 建议落点 | 依赖 | 验证方式

要求：
- 每个任务都使用动作导向表述，例如“新增……”“抽离……”“实现……”“校验……”。
- 优先给出最小可落地切片，而不是大而全的史诗任务。
- 验证方式应尽量贴合当前仓库，例如 `yarn build`、窄范围 Rust 校验，或功能性检查。

## 风险与开放问题
- 只列真正会阻塞排期或实现路径选择的问题。

## 建议下一步
- 给出最值得先做的 1 个实现切片，并说明为什么它是当前最小且正确的起点。