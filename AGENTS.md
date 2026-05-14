# Project Guidelines

## Project Context
- PatentMate is an early-stage Tauri 2 desktop app for the PatentScribe AI workflow. The product source of truth is [docs/PatentScribe-AI-SPEC-V5.md](docs/PatentScribe-AI-SPEC-V5.md).
- The current codebase is still close to the default scaffold. Prefer changes that move the app toward the PRD instead of extending the demo greet flow.

## Architecture
- Frontend UI lives in [src](src) and is a React 18 + TypeScript + Vite app.
- Native desktop logic lives in [src-tauri/src](src-tauri/src). Add Tauri commands there and expose them through `invoke` from the frontend.
- Tauri configuration lives in [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json). Keep frontend build output and dev URL aligned with that file.
- Product documentation lives in [docs](docs). Link to existing docs instead of copying requirements into code comments or new markdown files.

## Build And Validation
- Use Yarn for Node tasks. This repo has a [yarn.lock](yarn.lock), and [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) calls `yarn dev` and `yarn build`.
- Install dependencies with `yarn install`.
- Frontend-only development: `yarn dev`.
- Desktop app development: `yarn tauri dev`.
- Frontend production build: `yarn build`.
- Native packaging: `yarn tauri build`.
- There is no test suite yet. Validate changes with the narrowest relevant command, usually `yarn build` for frontend work and targeted Rust checks when backend code changes.

## Conventions
- Keep cross-layer features split cleanly: React handles interaction state and rendering; Rust handles native capabilities, file processing, and backend validation rules.
- When adding a new Rust command, register it in `tauri::generate_handler!` and return serde-serializable payloads instead of UI-formatted strings.
- Respect the fixed Tauri dev ports defined in [vite.config.ts](vite.config.ts) and [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json). `strictPort: true` means port conflicts break local startup.
- For new product work, start from the PRD requirement and then implement the smallest slice that preserves the intended architecture.