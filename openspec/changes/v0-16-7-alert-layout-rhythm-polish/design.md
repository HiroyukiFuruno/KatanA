## Context

Alert/admonition spacing is controlled in vendored `egui_commonmark`, not in a Katana-only wrapper. The current vertical rhythm is heavier than intended around the title row and block margins.

Because this is renderer-owned layout, the cleanest change is to specify and patch it independently as `0.16.7`.

## Goals / Non-Goals

**Goals:**
- Tighten alert title/body spacing
- Restrain whole-block vertical margins
- Keep the patch localized to the vendored alert renderer
- Add layout assertions against nearby paragraphs/lists

**Non-Goals:**
- Rework global markdown spacing policy
- Change non-alert blockquote behavior unless needed for regression safety
- Modify preview highlight or split sync behavior

## Decisions

### 1. Patch the vendored alert renderer directly

Alert spacing will be changed in the vendored `egui_commonmark_backend::alert_ui` path rather than in a Katana-side container.

- Rationale: this keeps responsibility with the renderer that owns alert structure
- Alternative considered: inject spacing around rendered alert blocks in Katana's preview layer
  - Rejected: that would blur ownership and make future subtree updates harder to reason about

### 2. Use asymmetric title-row padding with restrained block margins

The title row will use slightly different top and bottom padding, and the block margin will stay intentionally compact relative to adjacent paragraphs and lists.

- Rationale: this directly matches the desired reading rhythm
- Alternative considered: only reduce outer margins and leave title padding unchanged
  - Rejected: that would not address the cramped title/body relationship

## Risks / Trade-offs

- [Risk] Vendored changes may conflict with future upstream sync work
  → Mitigation: keep the patch localized to spacing constants and cover it with fixture assertions
- [Risk] Normal blockquotes may regress indirectly
  → Mitigation: explicitly test nearby non-alert block types

## Migration Plan

1. Patch alert spacing in the vendored renderer
2. Add fixture/layout assertions for surrounding paragraphs and lists
3. Verify non-alert blockquote behavior does not regress
