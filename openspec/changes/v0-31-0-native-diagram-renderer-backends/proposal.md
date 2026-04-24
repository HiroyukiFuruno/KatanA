## Why

PR #236 identified the Windows setup burden around diagram rendering. The current implementation confirms the root cause:

- Mermaid rendering shells out to `mmdc`, which requires Node.js and Mermaid CLI.
- PlantUML rendering shells out to `java -jar plantuml.jar`, so a Java runtime is required even though KatanA can download the jar.
- README copy currently claims a lightweight native binary, while Mermaid/PlantUML preview still depends on external runtimes for common diagram workflows.

Rust library candidates now exist for both ecosystems. This change is worth planning because KatanA's product direction is native, view-first preview; removing runtime setup friction directly improves first-run preview reliability on Windows and macOS.

## What Changes

- Introduce diagram renderer backend adapters for Mermaid and PlantUML so KatanA can select Rust-native, external CLI, or fallback backends through a single contract.
- Evaluate Mermaid Rust candidates, with `merman` and `mermaid-rs-renderer` as primary candidates and `selkie-rs` as a secondary candidate.
- Evaluate PlantUML Rust candidates, with `plantuml-little` as the primary candidate and PlantUML server/client crates rejected for offline desktop preview.
- Add parity and quality gates before changing defaults.
- Prefer Rust-native backends when they meet compatibility and licensing gates; retain external `mmdc` / `plantuml.jar` fallback while the migration stabilizes.
- Keep this independent from v0.28.0 preview adapter migration, but implement it through the same loose-coupling direction where possible.

## Capabilities

### New Capabilities

- `diagram-renderer-backends`: Adapter-managed Mermaid and PlantUML backend selection, parity verification, and fallback behavior.

### Modified Capabilities

- `diagram-block-preview`: Mermaid and PlantUML preview rendering may use Rust-native backends without user-installed Node.js or Java when the selected backend passes compatibility gates.

## Impact

- `crates/katana-core/src/markdown/mermaid_renderer/*`
- `crates/katana-core/src/markdown/plantuml_renderer/*`
- `crates/katana-core/src/markdown/diagram.rs`
- `crates/katana-ui/src/preview_pane/*`
- README / setup docs that mention Node.js, `mmdc`, Java, and PlantUML jar requirements.
- Diagram integration tests, cache keys, theme propagation tests, and export tests.
