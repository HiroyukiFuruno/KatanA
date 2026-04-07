## Context

Katana already keeps a `PreviewPane` per tab via `TabPreviewCache`, but the current render lifecycle is still effectively active-tab-only. `cancel_inactive_renders()` aborts background preview work for every non-active tab, and `PreviewPane::abort_renders()` drops the receiver and sets the cancel token as soon as the user switches away. That means a tab can keep a `Pending` preview section even though the user expects the work to continue in the background.

The problem is broader than diagrams. Mermaid, PlantUML, and Draw.io use worker threads and a render channel, while extracted local-image sections currently become visible only when `show_local_image()` reads bytes and attaches a texture during active drawing. The current preview model does not explicitly distinguish "the background result is ready" from "the active preview has attached and drawn it," so tab revisit cannot reliably hydrate partially finished work.

## Goals / Non-Goals

**Goals:**
- Keep per-tab preview work alive across tab switches for Mermaid, PlantUML, Draw.io, and tab-owned image-backed preview sections
- Introduce explicit per-tab preview lifecycle state that separates loaded background results from drawn/attached visible state
- Reattach completed background work on tab revisit without forcing the user through a stale loading state or a needless rerender
- Cancel obsolete work only when the source changes, the tab closes, or the user explicitly invalidates the preview
- Add regression coverage for tab switching during unfinished preview work

**Non-Goals:**
- Redesign the underlying Mermaid, PlantUML, or Draw.io render backends
- Persist preview job state across application restarts
- Keep completed preview results alive after a tab is fully closed
- Rework unrelated preview hover/sync behavior beyond what is necessary for lifecycle continuity

## Decisions

### 1. Introduce a tab-scoped preview session lifecycle

Each tab will own a preview session that survives focus changes. That session will hold section-level lifecycle entries keyed by stable section identity, so background jobs are tied to the tab and source generation rather than to the active frame only.

- Rationale: `TabPreviewCache` already scopes preview panes by path, so the missing piece is lifecycle ownership rather than a completely new container
- Alternative considered: keep the existing `PreviewPane` shape and only stop calling `cancel_inactive_renders()`
  - Rejected: removing the cancel call alone would still leave no explicit way to distinguish reusable completed work from visible drawn state

### 1.5 Use per-section state and a derived tab summary

`is_loaded` and `is_drawn` will be stored per section inside the tab preview session, not as one coarse flag per tab. The tab may expose a derived summary such as "any pending" or "all drawn", but the source of truth is per-section lifecycle state.

- Rationale: a tab can contain mixed states, for example one finished diagram, one still-pending diagram, and one local image that is loaded but not yet drawn
- Alternative considered: keep only a tab-level pair of booleans
  - Rejected: tab-level booleans are too coarse to hydrate mixed-content previews safely

### 2. Track loaded and drawn readiness separately

The preview session will carry two explicit readiness concepts per section, matching the user's requested model: `is_loaded` and `is_drawn`. Internally these may be represented as fields or a small enum-backed struct, but the contract is:

- `is_loaded = true`: the background result or asset material needed for display has completed
- `is_drawn = true`: the active preview has attached that result to the visible pane and no hydration is pending
- `is_loaded = true && is_drawn = false`: the next time the tab becomes active, the preview must attach/draw the already completed result instead of restarting the work

- Rationale: this state split directly matches the observed failure mode and gives a deterministic rehydration trigger
- Alternative considered: a single status enum such as `Pending | Ready | Visible`
  - Rejected: that collapses two orthogonal concerns and makes invalidation/hydration transitions harder to express clearly in code reviews and tests

### 3. Stop canceling work on focus loss; cancel only on invalidation

Tab switches will no longer abort preview work solely because another tab became active. Cancellation will instead be tied to source-hash or generation changes, explicit refresh/invalidation, tab close, or workspace teardown.

- Rationale: the current abort-on-focus-loss behavior is the direct cause of the user-visible stuck loading problem
- Alternative considered: pause and later resume worker threads
  - Rejected: the current worker model is simpler if jobs are allowed to finish and then either hydrate or be discarded by generation mismatch

### 4. Use source generation and section identity to reject obsolete completions

Each preview session will stamp jobs and loaded results with a generation derived from the tab path and source hash, plus a stable section key derived from the section ordinal within the split preview result for that generation. Results from older generations must be ignored after source edits, refreshes, or tab closure.

- Rationale: once background jobs are allowed to outlive tab focus, stale completions become the main correctness risk
- Alternative considered: match results only by section kind/source text as today
  - Rejected: repeated identical section payloads and mid-edit refreshes make that too weak for safe reuse

### 4.5 Allow lazy observation of background completion

Hidden tabs do not need a separate per-frame polling loop. It is acceptable for a hidden tab's completed result to remain queued until the next activation, as long as the completion is preserved, recognized as `is_loaded = true`, and hydrated without restarting the work.

- Rationale: this keeps the implementation simple while still satisfying the user's expectation that background work continues
- Alternative considered: actively poll every hidden tab every frame
  - Rejected: that would add unnecessary complexity and background churn for no user-visible benefit

### 5. Split image asset completion from visible texture attachment

For tab-owned image-backed preview sections, the session layer will own the asset completion state and the active preview will own the final texture attachment/draw step. Because the current local-image path reads bytes and loads textures during `show_local_image()`, this work must be moved out of the active-draw-only path so a tab can revisit a loaded-but-not-drawn result.

- Rationale: images and diagrams share the same user expectation even though their current pipelines differ
- Alternative considered: only solve worker-thread diagrams and leave images on the current active-draw path
  - Rejected: the request explicitly includes image-backed sections, and inconsistent lifecycle behavior across preview block types would remain confusing

### 5.5 Scope image continuity to tab-owned sections, with mandatory validation of the CommonMark/HTTP path

This change formally guarantees lifecycle continuity for image paths that are owned by the tab preview session in the current architecture, starting with extracted `RenderedSection::LocalImage` sections. During implementation, the engineer must also validate the CommonMark/HTTP image path under the same tab-switch reproduction; if it exhibits the same stuck-loading regression, it must be enrolled into the same lifecycle within this change rather than deferred silently.

- Rationale: `LocalImage` is already a first-class preview section, while HTTP/CommonMark images currently flow through different loader plumbing
- Alternative considered: declare all image paths solved without distinguishing ownership
  - Rejected: that would leave the implementation path ambiguous for another agent

## Risks / Trade-offs

- [Risk] Keeping inactive-tab work alive increases memory and background CPU usage
  → Mitigation: scope retained state to open tabs only, evict on close, and reuse source-hash generations to avoid duplicate work
- [Risk] Stale completions may attach to a newer source version
  → Mitigation: require generation/source-hash checks before any loaded result transitions to drawn
- [Risk] `is_loaded` / `is_drawn` can become inconsistent if updated from multiple code paths
  → Mitigation: centralize all state transitions in the preview session lifecycle layer and cover them with focused tests
- [Risk] Image loading may still depend partly on `egui::Context` and texture APIs that cannot run fully off-thread
  → Mitigation: keep background completion at the asset/result level and reserve only the final attach step for the active UI thread
- [Risk] Another implementer may misread the image scope as "only local images" or "all HTTP images are already covered"
  → Mitigation: explicitly require validation of the CommonMark/HTTP path during Task 3.1 and enrollment into the same lifecycle if the bug reproduces there

## Migration Plan

1. Introduce tab preview session state and generation tracking alongside the existing per-tab preview cache
2. Replace focus-loss cancellation with invalidation-driven cancellation
3. Move diagram jobs onto the new session lifecycle and hydrate completed results on tab revisit
4. Move local image-backed preview sections onto the same loaded-vs-drawn contract and validate the CommonMark/HTTP image path in the same branch
5. Add unit/integration tests for tab switching, background completion, hydration, and invalidation

Rollback can disable the new session lifecycle and restore the previous active-tab-only cancellation behavior, although that would reintroduce the stuck loading problem.
