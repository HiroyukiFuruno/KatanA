## Context

Katana is a greenfield desktop application intended to support spec-driven development on local workspaces. The request defines a macOS-first MVP in Rust with `egui` for the interface, `comrak` for Markdown processing, and architectural constraints that require local-first behavior, AI extensibility, plugin seams, and cross-platform expansion readiness from the start.

The MVP success criteria are narrow but strict: a workspace can be opened, Markdown can be edited, preview rendering works correctly, and common Markdown diagram blocks are rendered as diagrams rather than dumped as raw source. The design therefore needs to keep the first implementation small while preserving clean boundaries for later phases such as AI assist, richer editor internals, and external plugins.

## Goals / Non-Goals

**Goals:**
- Deliver a macOS desktop architecture that supports the MVP flows of workspace navigation, Markdown authoring, and live preview.
- Render `Mermaid`, `PlantUML`, and `Draw.io` blocks in the default preview experience because spec-driven Markdown loses value if common diagram syntax is not visualized.
- Separate the system into UI, core, and platform layers so filesystem, settings, rendering, and future integrations can evolve independently.
- Define AI provider and plugin extension seams that can be implemented incrementally without changing the editor or workspace model.
- Define a generic open source repository security baseline that can be enabled on a public GitHub repository without embedding maintainer-specific credentials or personal contact details in versioned planning artifacts.
- Prefer implementation choices that can be validated locally and extended to Windows and Linux later.

**Non-Goals:**
- Implement real-time collaboration, cloud sync, multi-user SaaS features, or any remote-first storage model.
- Build full AI assistant workflows in the MVP beyond the provider abstraction and orchestration boundaries.
- Support externally distributed dynamic plugins in the first delivery; the MVP only needs internal extension hooks and registration contracts.
- Introduce a rope-based editor engine in the MVP; large-file optimization is deferred until the current text-buffer approach is proven insufficient.
- Auto-save source Markdown files without explicit user intent; MVP saves to source files only on an explicit save action.
- Provide crash recovery or session restoration beyond preserving dirty state during the live process lifetime.

## Definition of Ready

Implementation work for this change starts only after the following conditions are met:
- Repository bootstrap and account-specific Git/GitHub setup are handled outside this change in a temporary operational runbook, and that runbook is removed after setup is complete.
- Diagram input formats are fixed to raw fenced `mermaid` source, raw fenced `plantuml` source containing `@startuml` and `@enduml`, and raw uncompressed fenced `drawio` XML containing `<mxfile>` or `<mxGraphModel>`.
- The PlantUML runtime path is fixed to a fully local renderer bundle that ships with the app distribution and does not depend on a hosted service.
- The preview surface decision remains HTML/SVG-capable so diagram output can be rendered inline without redesigning the shell.

## Definition of Done

Work on this change is done only when all of the following are true:
- The workspace, editor, preview, and built-in diagram flows operate from a sample spec-driven workspace on macOS.
- Automated tests cover workspace loading, unsaved buffer preview sync, Mermaid rendering, PlantUML rendering, Draw.io rendering, and fallback behavior for invalid diagram blocks.
- The application remains usable when no AI provider is configured and when a bundled plugin fails to initialize.
- Implementation notes document any runtime assets bundled for diagram rendering and the operational limits of the MVP.
- Built-in plugin registration is static and compile-time only, with no unresolved runtime manifest format left for the MVP.
- The repository includes a generic `SECURITY.md`, Dependabot configuration, and the required GitHub security settings for public OSS operation.

## Decisions

### 1. Use a layered module layout aligned to the product architecture

Katana will be organized around three top-level layers:
- `ui`: egui application shell, panes, menu wiring, and presentation state.
- `core`: document model, workspace model, markdown pipeline, preview coordination, AI orchestration interfaces, and plugin contracts.
- `platform`: filesystem access, settings persistence, keybindings, clipboard, and OS-specific integrations.

This keeps platform concerns out of the editor core and makes later cross-platform ports primarily a platform-layer problem. The main alternative was a feature-first flat module structure, but that would mix OS integration, rendering, and domain logic too early in a greenfield codebase.

### 2. Treat the workspace as the system root and keep documents local-first

The application will open a user-selected directory as a workspace root. The project tree, active document, preview, and future AI features will all resolve paths relative to that root. Markdown document buffers will load from the local filesystem, track dirty state in memory, and save back to disk explicitly.

This matches the request's project-oriented model and avoids inventing an application-specific storage backend. The alternative was to store documents in an internal database for session recovery and indexing, but that would complicate local-first behavior before the MVP proves the basic workflow.

### 3. Use an egui split-pane shell with a single shared application state

The MVP UI will use a three-pane layout: workspace tree, editor, and preview, with room reserved for a future AI side panel. A single application state container will own the active workspace, document buffers, preview state, and extension registries; UI components will render from that state and dispatch actions back into the core layer.

`egui` is the right MVP choice because it shortens iteration time and keeps the rendering model simple for a desktop prototype. The main alternative was `iced`, which offers a more retained-mode architecture, but it adds more setup cost for an MVP whose main risk is product validation rather than UI sophistication.

### 4. Render preview from the active in-memory buffer through a Markdown pipeline

The preview must reflect the latest editor buffer, not only persisted file contents. The core markdown module will parse the active document with `comrak`, produce preview-ready render output, and expose structured errors so the UI can show fallback states without crashing.

Using `comrak` gives GitHub Flavored Markdown compatibility with a mature Rust ecosystem. The alternative was building a custom parser or combining multiple crates, which would add maintenance risk without improving the MVP outcome.

### 5. Treat diagram blocks as first-class preview content

The preview pipeline will detect supported diagram blocks and route them to built-in renderer adapters before final presentation. The MVP-supported block types are `mermaid`, `plantuml`, and `drawio`, and the standard preview experience will render them inline by default instead of showing raw fenced code.

This is required because Katana is positioned as a spec IDE, and diagram-heavy Markdown is a normal authoring case rather than an edge case. The main alternative was to defer these renderers to later plugins, but that would leave the base product unable to visualize common Markdown-delivered design artifacts.

### 6. Use an HTML/SVG-capable preview surface for rich Markdown rendering

The preview surface will support HTML and SVG-capable output so that bundled diagram renderer assets can display inline results. `Mermaid` and `Draw.io` renderers can be hosted as bundled preview assets, while `PlantUML` output should be normalized into preview-safe SVG before display.

The alternative was a pure egui-widget preview implementation, but that would make standards-grade diagram rendering materially harder in the MVP and would push core Markdown visualization behind UI limitations.

### 7. Keep repository bootstrap outside product planning artifacts

Repository bootstrap, account selection, and GitHub remote creation are operational concerns rather than product requirements. If these steps need to be documented for execution, they should live in a temporary, non-product runbook that can be deleted after setup rather than inside versioned OpenSpec artifacts.

This keeps personal or environment-specific details out of the product definition and makes the architecture portable across maintainers and future repository ownership changes.

### 8. Fix the local PlantUML runtime strategy now

PlantUML rendering will use a bundled local runtime path: Katana will ship `plantuml.jar` and a bundled Graphviz `dot` binary for macOS inside the app distribution, invoke them through a dedicated adapter within the preview pipeline, and normalize output to SVG before display. MVP support is limited to fenced `plantuml` source blocks that contain explicit `@startuml` and `@enduml` delimiters.

This resolves the readiness gap in the previous plan and keeps the product aligned with the local-first requirement. The alternative was to depend on an external PlantUML service or a user-managed runtime, but that would either violate the offline requirement or make default preview behavior unreliable.

### 9. Keep source saving explicit and defer recovery features

The MVP will not auto-save source files and will not implement crash recovery or session restore. Dirty buffers live in memory until the user explicitly saves or closes the application, which keeps file writes predictable during the first release.

This removes ambiguity about document persistence behavior and keeps the authoring flow aligned with standard editor expectations for an early desktop tool. The alternative was adding snapshot recovery immediately, but that would introduce hidden state, lifecycle complexity, and more validation work before the base editing experience is proven.

### 10. Use static built-in plugin registration for the MVP

Built-in plugins will be declared through compile-time Rust registrations rather than runtime manifests or external descriptor files. The plugin registry will assemble these built-in registrations during startup and apply version checks against the internal plugin API contract.

This resolves the remaining manifest ambiguity and keeps plugin behavior deterministic for the first release. The alternative was adding a lightweight manifest format now, but that would expand configuration surface without a concrete third-party plugin need.

### 11. Stabilize AI integration behind provider traits and orchestration commands

The core AI module will expose provider traits for request execution, provider metadata, and error normalization. The rest of the app will depend only on this abstraction and a provider registry keyed by provider identifier. Provider-specific authentication, transport, and model details will remain encapsulated in provider adapters.

This allows future OpenAI, Claude, Gemini, or Ollama support without rewriting editor or workspace logic. The alternative was to defer AI structure entirely until Phase 2, but that would likely force later changes across the app once AI panels and prompt workflows are introduced.

### 12. Define plugin hooks now, but limit MVP loading to internal registrations

The plugin foundation will provide typed extension points for renderer enrichments, AI tools, and UI panels. MVP plugins will be statically linked or otherwise bundled with the application, registered during startup, and version-checked against a simple plugin API contract.

This preserves extensibility without taking on the complexity of dynamic library loading, sandboxing, or third-party distribution. The alternative was full runtime plugin loading, but that introduces security and compatibility risks that are not required to validate the MVP.

### 13. Treat OSS repository security as a first-class delivery concern

Before Katana is published as an open source repository, the project will define a repository security baseline that covers vulnerability reporting, dependency monitoring, code scanning, secret protection, and hardened GitHub Actions defaults. This baseline is generic to the repository and should avoid maintainer-specific credentials or personal contact details in tracked files.

The baseline will include:
- a versioned `SECURITY.md` that points reporters to GitHub private vulnerability reporting
- dependency graph, Dependabot alerts, and Dependabot security updates
- CodeQL default setup for public repository scanning
- secret scanning and push protection where available for public repositories
- a default branch ruleset requiring pull requests and status checks
- GitHub Actions hardening through restricted `GITHUB_TOKEN` defaults, approval for forked workflow runs, and pinned or explicitly trusted actions

The alternative was to treat repository security as a post-launch administrative concern, but that would leave the public project in a weaker state exactly when outside contributions begin.

## Risks / Trade-offs

- [Immediate-mode UI state can sprawl as features grow] -> Mitigation: keep domain state in the core layer and restrict egui code to presentation and event dispatch.
- [Text-buffer editing may degrade on large files] -> Mitigation: isolate editor state behind a document interface so a rope-based backend can replace it later.
- [Users may expect autosave or recovery in a desktop editor] -> Mitigation: document explicit-save behavior clearly in the MVP and revisit recovery after the base editing flow is validated.
- [Cross-platform readiness may be eroded by macOS-specific shortcuts or integrations] -> Mitigation: route platform services through traits and keep OS-specific code inside the platform layer.
- [Preview performance may lag if full re-render runs on every keystroke] -> Mitigation: introduce change coalescing and incremental refresh thresholds if profiling shows visible regressions.
- [Bundling multiple diagram renderers can increase packaging and sandbox complexity] -> Mitigation: isolate diagram renderers behind adapters with explicit asset/runtime requirements and keep a graceful source fallback for renderer failures.
- [Account-specific bootstrap instructions can leak personal details into planning artifacts] -> Mitigation: keep Git and GitHub setup in a temporary operational note outside versioned product docs.
- [Public OSS exposure increases supply-chain and disclosure risk] -> Mitigation: ship with a repository security baseline and enable GitHub's public-repo security features before opening broad contributions.
- [Future AI providers may require incompatible auth or streaming semantics] -> Mitigation: model provider capabilities explicitly in the provider interface instead of assuming a single synchronous request shape.
- [Plugin hooks can become too generic or too rigid] -> Mitigation: define a small set of extension points tied to concrete use cases and version the plugin API contract from the start.

## Migration Plan

This is a greenfield change, so no production migration or rollback sequence is required. Implementation should land incrementally behind compile-ready modules and local validation so unfinished AI or plugin adapters do not block the core editor workflow.

## Open Questions

- None for MVP planning. Remaining unknowns belong to implementation spikes or later-phase product changes rather than this change definition.
