# PatentMate

PatentMate 是一个基于 **Tauri 2** 的桌面应用，用于承载 **PatentScribe AI** 工作流。

当前仓库仍处于早期阶段，代码整体接近默认 scaffold。后续开发应优先朝产品需求推进，而不是继续扩展当前的 demo `greet` 流程。

## 项目定位

- **产品名称**：PatentMate
- **目标场景**：PatentScribe AI 专利交底书优化工作流
- **产品需求源**：`docs/PatentScribe-AI-SPEC-V5.md`
- **当前状态**：早期桌面端骨架，前后端基础链路已建立，业务能力尚未落地

## 技术栈

- **Frontend**: React 18 + TypeScript + Vite
- **Desktop Shell**: Tauri 2
- **Native / Backend**: Rust
- **Package Manager**: Yarn

## 仓库结构

```text
.
├─ docs/                     # 产品文档，需求以此为准
├─ src/                      # React 前端
├─ src-tauri/                # Tauri 配置与 Rust 原生逻辑
│  └─ src/                   # Tauri commands / native backend
├─ public/                   # 前端静态资源
├─ package.json              # Node 脚本与依赖
├─ vite.config.ts            # Vite 配置（固定开发端口）
├─ README.md
└─ yarn.lock
```

## 当前实现情况

当前代码库仍基本保留默认 Tauri + React 模板内容：

- 前端仍是默认欢迎页
- Rust 侧仍只暴露示例 `greet` command
- `src-tauri/tauri.conf.json` 已接入 Yarn 工作流

这意味着后续开发的重点应是：

- 建立 PatentScribe AI 的真实界面结构
- 打通前端与 Tauri command 的业务链路
- 围绕 PRD 增量实现文档处理、对话流程、预览与导出能力

## 开发约定

### 前后端职责划分

- **React**：负责交互、状态、界面渲染
- **Rust / Tauri**：负责原生能力、文件处理、后端校验规则

### Tauri command 约定

新增 Rust command 时应：

- 在 `tauri::generate_handler!` 中注册
- 返回 **serde 可序列化** 的结构化数据
- 避免返回只适合 UI 展示的格式化字符串

### 配置约定

当前开发端口为：

- Vite: `1420`
- Vite HMR: `1421`

`vite.config.ts` 与 `src-tauri/tauri.conf.json` 需要保持一致；项目开启了 `strictPort: true`，端口冲突会直接导致启动失败。

## 快速开始

### 1. 安装依赖

````bash
yarn install
````

### 2. 启动前端开发环境

````bash
yarn dev
````

### 3. 启动桌面应用开发环境

````bash
yarn tauri dev
````

## 构建命令

### 前端生产构建

````bash
yarn build
````

### 桌面应用打包

````bash
yarn tauri build
````

## 验证建议

当前仓库还没有测试套件，建议按改动范围做最小验证：

- **前端改动**：运行 `yarn build`
- **Tauri / Rust 改动**：做对应的 Rust 定向检查
- **桌面联调**：运行 `yarn tauri dev`

## 开发方向建议

如果要开始真正的产品开发，建议优先完成这些最小切片：

1. 将默认欢迎页替换为 PatentScribe AI 的基础布局
2. 建立左侧对话区 / 右侧预览区的页面骨架
3. 用结构化 Tauri command 替换示例 `greet`
4. 为文档输入、分析结果、预览内容定义稳定的数据模型

## 参考文档

- 产品需求：`docs/PatentScribe-AI-SPEC-V5.md`
- Tauri 配置：`src-tauri/tauri.conf.json`
- 前端配置：`vite.config.ts`

## 推荐开发环境

- [Visual Studio Code](https://code.visualstudio.com/)
- [Tauri VS Code Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
