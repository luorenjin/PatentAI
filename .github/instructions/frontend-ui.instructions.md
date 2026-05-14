---
description: "Use when building or refactoring the PatentScribe React UI, chat panel, preview panel, highlight annotations, message composer, or frontend state flow."
name: "PatentScribe Frontend UI"
applyTo: ["src/**/*.tsx", "src/**/*.css"]
---
# PatentScribe Frontend UI

- Use [docs/PatentScribe-AI-SPEC-V5.md](docs/PatentScribe-AI-SPEC-V5.md) as the UX source of truth for the AI-native interface. Link to it instead of copying long requirement lists into code or new markdown files.
- Preserve the primary interaction model: left-side conversation workspace, right-side live disclosure preview, and session actions such as download current version or stop early remaining reachable during the workflow.
- Model chat messages explicitly by role and intent, such as system status, AI diagnosis, AI follow-up question, engineer answer, and error state, so bubble rendering and action availability do not depend on ad-hoc string checks.
- Keep the preview driven by one canonical disclosure document state plus annotation metadata. Do not let rendered preview content and source document content drift into separate sources of truth.
- Represent AI-added content and AI-rewritten content as distinct annotation types from the state model upward. The default visual language should stay consistent with the PRD: blue for AI additions and yellow for AI modifications.
- Treat preview refresh as part of the answer cycle: submit or skip answer, show pending state, update preview, then render the next follow-up or completeness suggestion. Prefer explicit stage-based state over many loosely related booleans.
- Separate durable session state, such as messages, answers, draft versions, annotations, and settings, from transient UI state, such as input text, loading flags, active panel controls, and zoom level.
- As the UI grows, extract feature-focused components instead of expanding a single root component. Keep conversation timeline, composer, preview panel, preview controls, and annotation legend as separable responsibilities even if file names differ.
- For layout and styling work, optimize for long-form review: stable scroll regions, readable content width, clear panel hierarchy, and desktop-first split view that still collapses cleanly on narrow widths.