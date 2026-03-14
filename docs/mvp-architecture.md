# Katana MVP Architecture

This document describes the MVP architecture, constraints, and follow-on expectations.

## Module Layout

```
crates/
  katana-core/       — Document model, workspace model, Markdown pipeline,
  │                    AI provider abstraction, plugin contracts
  ├─ src/
  │   ├─ ai/         — AiProvider trait, AiProviderRegistry
  │   ├─ markdown/   — comrak pipeline + diagram block detection/routing
  │   │   └─ diagram.rs  — DiagramKind, DiagramBlock, DiagramRenderer trait, NoOpRenderer
  │   ├─ plugin/     — PluginRegistry, ExtensionPoint, PLUGIN_API_VERSION
  │   ├─ document.rs — Document, DocumentError
  │   └─ workspace.rs — Workspace, TreeEntry, WorkspaceError

  katana-platform/   — Filesystem access, settings persistence, OS integration
  ├─ src/
  │   ├─ filesystem.rs  — FilesystemService (open_workspace, load_document, save_document)
  │   └─ settings.rs    — SettingsService (MVP: in-memory only)

  katana-ui/         — egui application shell, panes, action dispatch
  ├─ src/
  │   ├─ app_state.rs — AppState, AppAction
  │   ├─ shell.rs     — KatanaApp (eframe::App impl), three-pane layout
  │   └─ main.rs      — Entry point, service init, plugin registration
```

## Explicit-Save Constraint

The MVP does **not** auto-save source Markdown files. Disk writes occur only
when the user explicitly triggers a save action. In-memory buffers track dirty
state independently of the on-disk content. This is a deliberate design choice
to keep file writes predictable during the first release.

## Diagram Rendering Constraints

MVP-supported input formats:

| Block type | Required payload format |
|------------|------------------------|
| `mermaid`  | Raw Mermaid source text |
| `plantuml` | Raw PlantUML source with `@startuml` / `@enduml` delimiters |
| `drawio`   | Raw uncompressed XML containing `<mxfile>` or `<mxGraphModel>` |

Unsupported encodings (compressed XML, base64, external references) are
rejected at validation and sent through the diagram fallback path, which
renders the source as a code block with an error label.

## Bundled Runtime Assets

| Asset | Purpose | Status |
|-------|---------|--------|
| Built-in Mermaid renderer | Render `mermaid` fences | Task 4.2 — placeholder registered |
| `plantuml.jar` + `dot` binary | Render `plantuml` fences locally | Task 4.4 — follow-on |
| Built-in Draw.io renderer | Render `drawio` fences | Task 4.3 — placeholder registered |

> **Note**: Mermaid and Draw.io rendering requires a WebView surface (Task 3.2).
> PlantUML requires bundling `plantuml.jar` and Graphviz `dot` for macOS (Task 4.4).
> Both are follow-on implementation tasks.

## Plugin API Contract

- `PLUGIN_API_VERSION = 1`
- All MVP plugins are registered statically at startup in `main.rs`.
- No runtime manifest file is required.
- Plugins that fail initialization are disabled; the application continues.

## Follow-On Implementation Expectations

| Phase | Description |
|-------|-------------|
| Task 3.2 | Integrate WebView surface for HTML/SVG preview |
| Task 4.2 | Wire Mermaid renderer into diagram pipeline |
| Task 4.3 | Wire Draw.io renderer into diagram pipeline |
| Task 4.4 | Bundle `plantuml.jar` + `dot` and wire PlantUML adapter |
| Phase 2  | Real AI provider adapters (OpenAI, Claude, Gemini, Ollama) |
| Phase 2+ | External plugin loading, sandboxing, and third-party distribution |
| Phase 3+ | Rope-based editor engine for large-file performance |
| Phase 3+ | Crash recovery and session restore |
