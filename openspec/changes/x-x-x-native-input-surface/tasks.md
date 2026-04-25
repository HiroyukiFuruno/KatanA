## Branch Rule

This change is version-undecided and deferred.
- **Standard Base Branch**: `x-x-x-native-input-surface` or a future versioned release branch once scheduled
- **Working Branch**: `x-x-x-native-input-surface-task-x` before scheduling, or `feature/vX.Y.Z-task-x` after scheduling

Do not implement this as part of v0.22.5 stabilization. Do not assign a release version until a future prioritization pass decides that the implementation cost is worth paying. When implementation is scheduled, use the normal OpenSpec delivery workflow for each task slice.

## 1. Scope Cleanup and Change Boundary

### Definition of Done (DoD)

- [ ] Confirm v0.27.0 and v0.29.0 remain valid preview-first/local-correction approaches to reducing code-centric editing pressure.
- [ ] Confirm this change records the source-input replacement option discovered during v0.22.5 input strengthening, not a scheduled release commitment.
- [ ] Add cross-reference notes to v0.27.0 and v0.29.0 artifacts if those changes still imply ownership of this work.
- [ ] Confirm no React, WebView, DOM runtime, or bundled web app direction is introduced for this input-surface problem.
- [ ] Confirm release/version assignment remains blocked until value and priority are explicitly re-evaluated.

- [ ] 1.1 Review `openspec/changes/v0-27-0-decouple-editor` for wording that conflates editor boundaries with source-input replacement.
- [ ] 1.2 Review `openspec/changes/v0-29-0-preview-driven-local-editing` for wording that conflates preview-local editing with full input-surface ownership.
- [ ] 1.3 Keep the native input surface as a deferred option, not as a required task for v0.27.0 or v0.29.0.

## 2. Prioritization Checkpoint

### Definition of Ready (DoR)

- [ ] Task 1 is complete and scope boundaries are clear.

### Definition of Done (DoD)

- [ ] Decide whether source-input emoji correctness still requires a custom input surface after preview-local editing progress is known.
- [ ] Assign a concrete release version only if the work is judged worth doing.
- [ ] Keep this change unscheduled if the preview-first editing direction sufficiently reduces the source-input problem.

- [ ] 2.1 Compare expected user value against implementation cost for IME, emoji, accessibility, and test coverage.
- [ ] 2.2 Decide whether this change remains deferred, gets versioned, or is closed as unnecessary.

## 3. Input Model Architecture

### Definition of Ready (DoR)

- [ ] Task 2 assigns a concrete release version and confirms implementation should proceed.
- [ ] The current source-edit path remains stable and verified.

### Definition of Done (DoD)

- [ ] Define input model ownership for buffer edits, cursor, selection, composition, undo/redo, clipboard, and pending authoring operations.
- [ ] Ensure the model does not use `egui::TextEdit` state as the source of truth.
- [ ] Add unit tests for text edits, selection edits, snippet insertion, undo/redo, and grapheme-aware cursor movement.

- [ ] 3.1 Design `InputBuffer`, `InputCursor`, `InputSelection`, `CompositionState`, and edit event types or equivalent abstractions.
- [ ] 3.2 Define how Markdown authoring commands apply edits through the input model.
- [ ] 3.3 Define dirty/save event emission from input-model changes.

## 4. Native Input Surface Prototype

### Definition of Ready (DoR)

- [ ] Task 3 input model is complete.
- [ ] The prototype can be enabled behind a feature flag or internal setting.

### Definition of Done (DoD)

- [ ] Implement a prototype input surface that renders and updates the input model without using `egui::TextEdit`.
- [ ] Preserve typing, cursor movement, selection movement, scroll/caret visibility, copy/cut/paste, undo/redo, and image paste.
- [ ] Keep fallback source editing available while the prototype is incomplete.

- [ ] 4.1 Implement pointer hit testing and caret placement.
- [ ] 4.2 Implement selection drag and keyboard selection movement.
- [ ] 4.3 Implement clipboard text paste and clipboard image paste through existing image-ingest behavior.
- [ ] 4.4 Implement scroll behavior and caret visibility without reintroducing forced-scroll regressions.

## 5. IME and Emoji Rendering Boundary

### Definition of Ready (DoR)

- [ ] Task 4 prototype input surface is available behind a gate.

### Definition of Done (DoD)

- [ ] Support IME composition lifecycle without corrupting buffer, cursor, selection, or preview synchronization.
- [ ] Isolate emoji/special-character rendering from egui `TextEdit` and document the chosen rendering boundary.
- [ ] Add platform coverage for macOS, Windows, and Linux where supported by CI/manual verification.

- [ ] 5.1 Add IME composition tests or harness coverage where automation is feasible.
- [ ] 5.2 Verify Japanese input behavior manually before enabling broadly.
- [ ] 5.3 Verify emoji and multi-codepoint grapheme hit testing.
- [ ] 5.4 Decide whether the rendering boundary uses custom egui painting, platform-native text views, or a dedicated layout/rasterization layer.

## 6. Integration and Rollout

### Definition of Ready (DoR)

- [ ] Tasks 3-5 are complete and fallback editing remains available.

### Definition of Done (DoD)

- [ ] Integrate the native input surface with full source-edit fallback mode and preview-driven local edit surfaces where appropriate.
- [ ] Verify preview sync, save/dirty semantics, diagnostics refresh, Markdown authoring controls, and accessibility behavior.
- [ ] Keep the old source input path until parity criteria are met.

- [ ] 6.1 Add UI integration tests for typing, selection, paste, image paste, authoring commands, scroll, and preview synchronization.
- [ ] 6.2 Add compatibility tests for fallback `TextEdit` path while the native surface is gated.
- [ ] 6.3 Define removal criteria for the old egui `TextEdit` input path.
- [ ] 6.4 Run `openspec validate x-x-x-native-input-surface` and the relevant Rust/UI checks before delivery.
