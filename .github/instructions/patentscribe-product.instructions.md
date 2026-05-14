---
description: "Use when implementing PatentScribe AI product features, AI chat workflow, document parsing, live preview, export, PDF handling, or Tauri commands tied to the PRD."
name: "PatentScribe Product Rules"
applyTo: ["src/**", "src-tauri/**"]
---
# PatentScribe Product Rules

- Treat [docs/PatentScribe-AI-SPEC-V5.md](docs/PatentScribe-AI-SPEC-V5.md) as the product source of truth for PatentScribe AI features. Link to it instead of duplicating long requirement lists.
- Preserve the V1 scope from the PRD: desktop only, Chinese first, no patent search, no claims generation, no multi-agent review, no mobile client, and no local deployment path.
- Keep the core interaction model intact: left-side conversational workflow, right-side live disclosure preview, engineer can stop early, and AI completeness judgments are advisory only.
- Maintain the PDF handling split defined in the PRD: default direct-to-model processing, optional local Rust preprocessing mode.
- Enforce the disclosure output contract when implementing generation logic: 8 fixed sections plus 2 special outputs. Missing information must be marked as `【待工程师补充】`; do not fabricate technical facts.
- Place output-format validation and retry logic in the Tauri/Rust layer when that feature is implemented, not only in the React UI.
- When scaffold behavior conflicts with the PRD, evolve the architecture toward the PRD instead of preserving template demo behavior.