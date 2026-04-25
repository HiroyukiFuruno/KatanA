## Context

Current Mermaid rendering uses `mmdc` and writes a temporary `.mmd` file, then asks Mermaid CLI to produce PNG. Current PlantUML rendering finds `plantuml.jar`, starts `java`, pipes source to stdin, and reads SVG from stdout. That model is functional but not aligned with a native desktop application that should avoid requiring Node.js and Java for the default preview path.

The candidate evaluation below was performed on 2026-04-24 using crates.io metadata and local crate README / source packages.

## 2026-04-25 master 同期

`diagram-backend-adapter` Task 1 が `9ffeb570 feat: 図表backend adapter契約を追加` として `master` に入った。Mermaid / PlantUML の backend input、render options、theme snapshot、document context、renderer-neutral output / error、cache key contract は既に `katana-core` 側に追加済みである。

この change では同じ契約型を再定義しない。以降の実装は、既存外部 Mermaid CLI / PlantUML jar backend をこの adapter contract の implementation へ移すこと、および Rust-native backend spike / default selection に集中する。

## Candidate Evaluation

| Target | Candidate | Version | Evidence | Assessment |
| --- | --- | --- | --- | --- |
| Mermaid | `merman` | 0.4.0 | Headless Rust Mermaid implementation pinned to Mermaid `@11.12.3`; `render` feature provides SVG rendering; `raster` adds PNG/JPG/PDF. | Strong primary candidate. Best fit for KatanA if SVG output integrates cleanly and parity is sufficient. |
| Mermaid | `mermaid-rs-renderer` | 0.2.2 | Pure Rust renderer, 23 diagram types, SVG/PNG support, claims large speedup over mermaid-cli. | Strong secondary candidate. Earlier-stage quality warning means it should be compared against KatanA fixtures before becoming default. |
| Mermaid | `selkie-rs` | 0.3.0 | Rust Mermaid parser/renderer with CLI, library API, PNG feature, and evaluation system. | Useful fallback candidate. README explicitly says active development, so default adoption needs a stricter parity gate. |
| Mermaid | `mermaid-text` | 0.16.1 | Pure Rust text / terminal rendering. | Not suitable for preview replacement because KatanA needs SVG/bitmap preview, not Unicode terminal diagrams. |
| PlantUML | `plantuml-little` | 1.2026.2-3 | Rust library + CLI, `convert(puml_source) -> SVG`, targets byte-exact SVG parity with Java PlantUML v1.2026.2 for supported types. | Strong primary candidate, subject to license review and Graphviz dependency packaging validation. |
| PlantUML | `plantuml-server-client-rs` / `plantuml-parser` | 0.6.2 | Client/parser for PlantUML Server. | Reject for default preview; it moves rendering to an external server instead of removing runtime dependency. |
| PlantUML | `mdbook-plantuml` | 2.0.0 | mdBook preprocessor with PlantUML server features. | Reject for KatanA preview; it is an mdBook integration and server path, not a local renderer backend. |

## Goals

- Remove Node.js and Java as required default runtime dependencies for Mermaid and PlantUML preview when Rust-native backends pass compatibility gates.
- Keep external CLI/JAR rendering as fallback until Rust-native rendering is proven across KatanA fixtures.
- Introduce backend adapter contracts so KatanA preview and export paths do not depend directly on a chosen crate.
- Preserve diagram cache, theme, export, and failure fallback semantics.
- Update user-facing setup docs to reflect the selected backend and remaining fallback requirements.

## Non-Goals

- Removing Draw.io support.
- Replacing every external backend in a single task without fixture parity.
- Introducing WebView, Node.js embedding, Deno, V8, or a React bundle.
- Depending on PlantUML server or remote rendering for default offline preview.
- Guaranteeing byte-identical output with `mmdc` or Java PlantUML for every upstream feature before exposing a guarded preview path.

## Decisions

### Backend Adapter Contract

Define a backend adapter for each diagram kind. A backend receives diagram source, theme snapshot, document context, and render options, then returns a renderer-neutral `DiagramResult`. The initial contract is already available on `master`; this change should migrate external and Rust-native implementations behind that contract instead of defining another parallel contract.

### Default Selection Is Gate-Driven

Rust-native backends may become the default only after they pass:

- supported diagram kind inventory;
- KatanA fixture parity tests;
- error/fallback behavior tests;
- theme propagation tests;
- export compatibility tests;
- license and packaging review.

### Mermaid Primary Candidate

Use `merman` as the first spike because it is explicitly parity-focused, has a library API, and can render SVG through Rust without browser startup. Compare it with `mermaid-rs-renderer` on KatanA fixtures before setting the default.

### PlantUML Primary Candidate

Use `plantuml-little` as the first spike because it exposes a direct `convert` library API and targets SVG parity with Java PlantUML. The adoption gate must include its multi-license choice and `graphviz-anywhere` packaging behavior on macOS, Windows, and Linux.

### Fallbacks Stay User-Recoverable

If a Rust-native backend cannot render a supported diagram, the system should either fall back to the configured external backend or display the existing recoverable diagram failure state. The migration must not collapse Markdown preview.

## Risks / Trade-offs

- **Risk: visual parity regressions** - Rust renderers may differ from upstream Mermaid or PlantUML. Mitigate with fixture parity snapshots and keep external backends available.
- **Risk: dependency packaging** - `plantuml-little` depends on Graphviz-related native packaging. Mitigate with a cross-platform spike before making it default.
- **Risk: dependency churn** - Mermaid Rust renderers are young. Mitigate by hiding them behind KatanA-owned backend adapters.
- **Trade-off: temporary dual backend support** - Keeping both Rust and external backends adds complexity, but it lets KatanA reduce runtime setup friction without risking total diagram regression.
