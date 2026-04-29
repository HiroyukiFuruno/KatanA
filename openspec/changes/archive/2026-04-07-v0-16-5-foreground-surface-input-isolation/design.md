## Context

Katana renders multiple classes of foreground UI: shell windows, modal dialogs, context menus, activity-rail menus, settings-local popups, and fullscreen/splash overlays. Today those surfaces do not share one authoritative blocker contract, so background-pane interaction can still leak through while the foreground surface is open.

The problem belongs to the shell interaction model rather than to any one modal implementation. `0.16.5` isolates that concern so another engineer or agent can implement one blocker registry and verify it independently.

## Goals / Non-Goals

**Goals:**

- Centralize foreground-surface blocking decisions in the shell
- Cover representative shell windows, popups, menus, and overlays
- Suppress background pane interaction without breaking the active foreground surface itself
- Add response-based UI tests for representative surfaces

**Non-Goals:**

- Redesign the modal framework
- Change tab pinning behavior
- Adjust markdown layout or preview mapping

## Decisions

### 1. The shell owns one blocker contract

The shell will determine whether the current frame contains an active foreground surface that should suppress background interaction. Individual widgets may still manage their own open state, but they will register into one shell-owned decision point.

- Rationale: this reduces gaps compared with duplicating `Sense` suppression across unrelated widgets
- Alternative considered: add a separate transparent overlay to each surface
  - Rejected: implementation would stay fragmented and leak-prone

### 2. Block background panes, not the foreground surface itself

When the blocker is active, workspace-tree, editor, preview, and tab-strip hover/click/context-menu reactions must stop updating. The foreground window, popup, or overlay remains fully interactive.

- Rationale: the user-facing bug is background bubbling, not broken foreground controls
- Alternative considered: globally suppress all pointer input
  - Rejected: that would break close, scroll, typing, and drag operations in the foreground surface

### 3. Cover blocker sources by explicit category

Implementation and tests will treat shell windows, context/popup menus, settings-local popups, and overlay/detached surfaces as explicit blocker-source categories.

- Rationale: another agent needs a concrete inventory to avoid under-scoping the fix
- Alternative considered: describe only "modals" generically
  - Rejected: current leakage is broader than traditional modal windows

## Risks / Trade-offs

- [Risk] The blocker may be too broad and suppress valid interaction
  → Mitigation: limit the suppression to background panes and verify foreground close/scroll/form input in tests

- [Risk] Some popup sources may still be missed
  → Mitigation: keep an explicit source inventory in tasks and tests

## Migration Plan

1. Add the shell blocker contract and source inventory
2. Wire representative windows, menus, popups, and overlays into that contract
3. Add UI tests for representative source categories
4. Run UI verification before delivery
