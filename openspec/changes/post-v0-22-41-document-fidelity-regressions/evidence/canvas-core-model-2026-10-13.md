# Read-only JSON Canvas model — #258

## Implemented boundary

The core module parses the [JSON Canvas 1.0 specification](https://jsoncanvas.org/spec/1.0/) into owned text/file/link/group nodes, styled edges, geometry and explicit unsupported-field/type diagnostics. Node order is retained. Parsing never opens a referenced file, downloads a URL, or executes active content. Missing top-level arrays become empty arrays. Malformed data, duplicate IDs within each namespace, missing endpoints and invalid geometry produce typed errors without echoing source content.

This is the model component of the full read-only viewer, not a functioning Canvas preview. File recognition, asynchronous host loading, cards/groups/edges rendering, unsupported-state display, pan/zoom/fit, translation and host input/preview regressions remain to implement.

## Main review corrections

- Initial helpers were refactored into private struct/impl boundaries before integration; no lint exemptions were added.
- A high-reasoning public-API review identified an unnecessary shared node/edge ID namespace. Main added an acceptance regression which failed at runtime (exit 101), then separated the sets.
- A unique empty ID was unnecessarily rejected. The acceptance regression failed at runtime, then the extra restriction was removed while duplicate rejection remained.
- A raw fractional JSON number was rounded into an integer before validation. The raw-literal rejection regression failed at runtime, then validation required an integer JSON number before conversion. Coordinates outside the model's exact integer precision range remain explicit typed errors rather than silently distorted positions.
- The first official AST run failed only on two ordinary comment annotations. They now use the existing WHY form in Japanese, preserving the latest human language instruction.

## Actual results

- Final core Canvas suite: 24 passed, zero failed/ignored, exit 0. `tmp/canvas-public-review-final-green-2026-10-13.log`.
- `cargo clippy --locked -p katana-core --all-targets -- -D warnings`: exit 0. `tmp/canvas-public-review-final-clippy-2026-10-13.log`.
- Official `just ast-lint`: 23 passed, exit 0. `tmp/canvas-public-review-final-ast-2026-10-13.log`.
- `cargo fmt --all` and `git diff --check`: exit 0.

Existing renderer/coverage/packaging gates are not replaced by these focused results. Issue #258 remains open until the complete viewer and its acceptance conditions are verified.
