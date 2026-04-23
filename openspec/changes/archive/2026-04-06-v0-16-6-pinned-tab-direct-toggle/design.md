## Context

Pinned tabs are rendered with a compact visual pin marker, but that marker is currently only status, not control. Unpin is still routed through the context menu, which conflicts with the visible affordance.

The work is focused on tab hit testing and action routing. `0.16.6` should therefore stay isolated from broader tab grouping or shell interaction work.

## Goals / Non-Goals

**Goals:**

- Make the visible pin icon a direct unpin toggle
- Keep tab activation separate from unpin action
- Preserve existing context-menu and ordering behavior
- Add one real interaction test for the icon click path

**Non-Goals:**

- Redesign tab grouping
- Change other context-menu actions
- Alter pinned-tab ordering rules

## Decisions

### 1. Split the hit targets

The tab body and pin icon will have distinct hit targets. Clicking the body activates the tab; clicking the icon toggles pin state.

- Rationale: activation and unpin are separate intents and should not share one click target
- Alternative considered: keep one hit target and infer intent from pointer position late
  - Rejected: that would stay fragile and harder to test

### 2. Keep context-menu pin flows intact

The direct icon toggle supplements, rather than replaces, the context-menu path.

- Rationale: existing workflows should not regress just because a faster path is added
- Alternative considered: remove context-menu unpin once direct toggle exists
  - Rejected: unnecessary behavior change

## Risks / Trade-offs

- [Risk] The icon target may be too small or overlap activation behavior
  → Mitigation: keep a dedicated hit target and cover it with an interaction test
- [Risk] Pin ordering or grouped-tab behavior may regress indirectly
  → Mitigation: verify direct toggle against existing ordering and grouping rules

## Migration Plan

1. Split pin-icon and tab-body hit targets
2. Route icon click to `TogglePinDocument`
3. Add interaction coverage for direct unpin
4. Verify no regression in context-menu pin/unpin behavior
