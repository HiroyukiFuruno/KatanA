## Context

Rich preview blocks do not behave like plain markdown text. Diagram blocks are replaced asynchronously after rendering, and alert/admonition blocks introduce structured block boundaries in preview. As a result, hover highlight and split sync cannot rely on the same assumptions as plain markdown sections.

`0.16.8` isolates the preview/source-mapping problem so another agent can implement one rich-block mapping contract without mixing it with shell blocking or alert spacing work.

## Goals / Non-Goals

**Goals:**
- Stabilize hover-highlight source mapping for Mermaid, PlantUML, Draw.io, and alert/admonition blocks
- Keep split sync aligned around rich-block boundaries
- Preserve stable source anchors across pending-to-rendered replacement for diagrams
- Add response-based integration coverage

**Non-Goals:**
- Redesign diagram render backends
- Change modal or tab behavior
- Adjust alert spacing constants

## Decisions

### 1. Use block-level source mapping as the single source of truth

Preview hover highlight and split sync will both read from one block-level source mapping contract for rich blocks.

- Rationale: fixing hover and sync separately tends to reintroduce drift
- Alternative considered: patch hover with rect heuristics and sync with independent anchor math
  - Rejected: that would not keep the two behaviors aligned

### 2. Keep diagram anchors stable across pending and rendered states

Rendered Mermaid, PlantUML, and Draw.io blocks will preserve source-line span and block identity across pending-to-rendered replacement.

- Rationale: async replacement is the main source of diagram-specific drift
- Alternative considered: rebuild source mapping from visible geometry only after render completion
  - Rejected: that is more fragile around replacement timing

### 3. Treat alert/admonition blocks as rich-block boundaries too

Alert/admonition blocks will participate in the same block-level mapping contract used by diagrams for hover and split sync.

- Rationale: the user explicitly wants alert notation covered in `0.16.8`
- Alternative considered: keep alerts on plain markdown mapping rules
  - Rejected: structured alert blocks can still distort boundary mapping

## Risks / Trade-offs

- [Risk] Anchor updates after async render completion may still produce jumps
  → Mitigation: test pre/post diagram boundaries and post-render convergence explicitly
- [Risk] Rich-block mapping may regress plain markdown sections indirectly
  → Mitigation: scope the new contract to rich-block paths and keep regression assertions focused

## Migration Plan

1. Introduce block-level source mapping for rich blocks
2. Stabilize diagram anchors across pending/rendered replacement
3. Route hover highlight and split sync through the shared mapping
4. Add integration coverage for diagrams and alert/admonition blocks
