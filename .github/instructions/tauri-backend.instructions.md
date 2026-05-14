---
description: "Use when adding or refactoring Tauri Rust commands, backend session state, document parsing, PDF or docx file handling, output format validation, or retry logic in src-tauri."
name: "PatentScribe Tauri Backend"
applyTo: ["src-tauri/src/**/*.rs", "src-tauri/build.rs", "src-tauri/Cargo.toml"]
---
# PatentScribe Tauri Backend

- Use [docs/PatentScribe-AI-SPEC-V5.md](docs/PatentScribe-AI-SPEC-V5.md) as the backend product source of truth. Link to it instead of copying long requirement blocks into Rust comments or new markdown files.
- Keep Rust responsible for native and authoritative workflows: file parsing, PDF mode selection, disclosure versioning, output-format validation, retry orchestration, export assembly, and backend-safe error shaping.
- Keep Tauri commands thin. Put parsing, validation, retry, and document transformation logic in plain Rust modules and call those modules from `#[tauri::command]` entrypoints.
- Define typed request and response payloads with serde instead of returning UI-formatted strings. Prefer enums and structs that let the frontend distinguish success, validation failure, retry exhaustion, unsupported format, and recoverable processing errors.
- Validate inputs at the command boundary before expensive work starts. Reject unsupported file types, malformed payloads, and impossible mode combinations with structured errors that are safe to surface in the UI.
- Model PDF handling explicitly. The default path is direct-to-model processing, while local preprocessing is an opt-in backend mode; do not bury that decision in scattered booleans or frontend-only state.
- Treat document processing as a pipeline with named stages, such as ingest, normalize, analyze, validate, regenerate, and export, so partial failures and retries stay diagnosable.
- Implement PRD output-format validation in Rust against the required 8 fixed sections, 2 special outputs, table requirements, Q-format requirements, and the `【待工程师补充】` rule for missing technical facts.
- Retry logic must be bounded and validation-driven. Follow the PRD expectation of at most 3 generation attempts, record the reason each attempt failed validation, and return a structured terminal failure when the retry budget is exhausted.
- Return machine-readable validation details, such as missing section identifiers or format-rule violations, so the frontend can render status and guidance without re-parsing backend messages.
- Register every new command in `tauri::generate_handler!` and keep shared backend state in explicit Rust models rather than ad-hoc globals.
- Prefer Rust-native crates that match the PRD architecture, and keep new dependencies focused on desktop-native concerns like parsing, validation, and document generation.