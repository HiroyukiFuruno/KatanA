## Why

Katana needs an implementation-ready plan that turns the product vision in `katana_openspec_request.md` into a bounded macOS MVP. Defining the MVP scope and extensible architecture now avoids rework later because local-first storage, cross-platform expansion, AI provider abstraction, and plugin seams all affect the initial app foundation.

## What Changes

- Define the macOS MVP around the core flows of opening a local workspace, editing Markdown documents, and viewing rendered preview output including standard diagram blocks.
- Establish a Rust-based layered architecture using `egui` for the UI shell and `comrak` for GitHub Flavored Markdown parsing/rendering.
- Specify the workspace model, application layout, and document lifecycle needed for spec-driven projects that contain files such as `spec.md`, `architecture.md`, and `tasks.md`.
- Make `Mermaid`, `PlantUML`, and `Draw.io` block rendering a built-in preview capability rather than an optional add-on.
- Add architectural requirements for an AI provider abstraction layer and a plugin foundation so future capabilities can be introduced without reshaping the editor core.
- Break the MVP into implementation tasks that can be executed incrementally, validated locally, and extended toward later phases.

## Capabilities

### New Capabilities
- `workspace-shell`: Open a local project workspace, show its file structure, and manage the active document within a three-pane desktop layout.
- `markdown-authoring`: Edit GitHub Flavored Markdown and render synchronized preview output suitable for spec-driven development documents.
- `diagram-block-preview`: Render `Mermaid`, `PlantUML`, and `Draw.io` blocks inline in the standard preview experience.
- `ai-provider-abstraction`: Define provider interfaces, configuration seams, and orchestration boundaries for future OpenAI, Claude, Gemini, and Ollama integrations.
- `plugin-foundation`: Define extension points and registration rules that allow future renderer, AI, and UI plugins without coupling them to the editor core.
- `oss-repository-security`: Establish the public repository security baseline for vulnerability reporting, dependency monitoring, code scanning, and hardened automation.

### Modified Capabilities
- None.

## Impact

- Introduces initial OpenSpec artifacts for Katana MVP planning.
- Affects the future Rust crate/module layout across UI, core, and platform layers.
- Commits the MVP to `egui` and `comrak`, built-in diagram rendering for common Markdown diagram blocks, and trait-based boundaries for AI and plugin integrations.
- Establishes the scope that later implementation and validation work will follow for the macOS desktop application.
