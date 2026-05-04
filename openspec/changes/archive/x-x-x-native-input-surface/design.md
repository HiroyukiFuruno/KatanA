## Context

KatanA is preview-first, but editable Markdown still needs a reliable source-input path for fallback editing, local preview edits, and structured authoring operations. The current source input path uses egui `TextEdit`, which means caret, selection, IME, undo/redo, clipboard, and text rendering behavior are tied to egui's text stack.

The motivation for this change came from the v0.22.5-era input strengthening work, before the later preview-driven local editing roadmap fully takes over. While fixing concrete input regressions, the team hit the older egui emoji limitation again and concluded that a separate input surface is the durable technical answer. The team also chose to defer whether that answer is worth implementing now.

v0.27.0 and v0.29.0 remain valid because they reduce reliance on code-centric full-document editing by moving correction workflows toward preview-local input. This change does not supersede that direction; it records the heavier fallback/source-input replacement idea as a deferred, version-undecided option.

## Goals / Non-Goals

**Goals:**

- Define a native input surface that owns Markdown text-entry state independently from egui `TextEdit`.
- Keep the input model reusable from full source-edit fallback and preview-driven local edit surfaces.
- Preserve normal Japanese and multilingual input, including IME composition, selection movement, clipboard text paste, image paste, undo/redo, and shortcut pass-through behavior.
- Establish an emoji-safe rendering boundary so the implementation can avoid egui's platform-specific emoji behavior for input text.
- Keep v0.27.0 and v0.29.0 focused on the preview-first local-correction approach by making this a separate deferred candidate.
- Preserve the reason for deferral: the architecture direction is known, but the value/prioritization decision is not yet made.

**Non-Goals:**

- Implement this inside v0.22.5 stabilization.
- Decide that the native input surface is worth implementing in a specific version.
- Convert KatanA to React, WebView, DOM, or a bundled web app.
- Replace every egui widget in the application.
- Make preview-driven local editing depend on a full-document WYSIWYG editor.
- Remove the fallback source-edit capability before a replacement input surface is stable.

## Decisions

### Decision: Separate input model from input surface rendering

The implementation MUST first introduce an input model that owns buffer mutation, cursor, selection, composition ranges, undo/redo, clipboard operations, and pending authoring edits without depending on `egui::TextEdit` state.

Alternatives considered:

- Keep extending `egui::TextEdit`: lowest immediate cost, but preserves the emoji/special-character coupling this change exists to remove.
- Replace the whole editor UI in one step: faster conceptually, but too risky for IME, scroll, and command compatibility.

### Decision: Treat egui as an optional host, not the text-entry owner

The first custom surface can still be hosted inside egui panels and painted through egui primitives, but it MUST NOT use egui `TextEdit` as the owner of text entry, caret, selection, or IME state. If egui text layout remains insufficient for emoji-safe input rendering, a later task may introduce a dedicated text layout/rasterization layer behind the input surface.

Alternatives considered:

- Immediate native OS text view embedding: potentially best platform fidelity, but introduces macOS/Windows/Linux embedding complexity and harder cross-platform test coverage.
- Immediate custom glyph renderer: maximum control, but high implementation cost and higher risk around shaping, bidi, and accessibility.

### Decision: Keep v0.27.0 and v0.29.0 boundaries explicit

v0.27.0/v0.29.0 should continue to pursue the preview-first, local-correction strategy. They may prepare boundaries or local input contracts that this change consumes later, but they MUST NOT be forced to deliver the full custom source-input surface.

Alternatives considered:

- Put the work under v0.27.0: mixes editor boundary preparation with a difficult source-input rewrite and hides the value decision.
- Put the work under v0.29.0: dilutes the preview-local editing approach, which is a strong direction precisely because it avoids making code editing the primary workflow.

### Decision: Defer value judgment separately from technical conclusion

The technical conclusion is that an egui-`TextEdit`-independent input surface is the durable workaround if KatanA must solve emoji/special-character input at the source-editor layer. The product/value conclusion is intentionally unresolved: this change records the option without assigning a version or requiring implementation.

Alternatives considered:

- Drop the idea entirely: loses useful architectural context already discussed while debugging input and emoji issues.
- Assign it to the next version: overcommits to a large rewrite before proving that preview-local editing will not make the source-input problem less important.

## Risks / Trade-offs

- [Risk] IME behavior regresses for Japanese and other composition-based input. -> Mitigation: make IME composition state a first-class model concept and require platform/manual tests before replacing `TextEdit`.
- [Risk] Emoji rendering improves but cursor hit testing becomes inaccurate. -> Mitigation: require grapheme-aware layout metrics and cursor mapping tests before enabling the surface broadly.
- [Risk] The custom input surface duplicates v0.27.0 editor component work. -> Mitigation: v0.27.0 owns component extraction; this change owns `TextEdit` replacement and references v0.27.0 as a dependency candidate, not a subtask.
- [Risk] Preview-driven local edit surfaces in v0.29.0 choose incompatible input primitives. -> Mitigation: define a shared input-surface contract that local edit surfaces can adopt later.
- [Risk] Accessibility and OS text services lag behind egui `TextEdit`. -> Mitigation: keep fallback `TextEdit` or source edit path available until accessibility and platform behavior are verified.
- [Risk] The deferred change is mistaken for a scheduled roadmap commitment. -> Mitigation: keep the `x-x-x` versionless name, explicit deferral language, and a future prioritization checkpoint before implementation.

## Migration Plan

1. Keep v0.22.5 on the repaired egui `TextEdit` path.
2. Use v0.27.0 to establish editor crate/component boundaries without implementing the custom input surface.
3. Use v0.29.0 to define preview-driven local edit contracts without making it responsible for full source-input replacement.
4. Revisit this version-undecided change only after a future prioritization decision confirms that source-input emoji correctness is worth the implementation cost.
5. Gate rollout behind a feature flag or internal setting until parity with the old source input path is verified.

## Open Questions

- Should the first custom surface use egui painting with a custom text layout cache, or a platform-native text view abstraction?
- What is the minimum emoji-safe rendering target for macOS, Windows, and Linux before the old `TextEdit` path can be retired?
- Which accessibility APIs are required for the first release of the custom input surface?
- How much of the input model should live in `katana-editor` versus a smaller lower-level crate?
