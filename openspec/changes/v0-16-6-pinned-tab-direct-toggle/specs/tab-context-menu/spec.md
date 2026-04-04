# Delta Spec: Tab Context Menu — Pin Toggle Affordance

## MODIFIED Requirements

### Requirement: ピン留めタブ

Pinned tabs can be fixed at the front of the tab strip. Pinned tabs use a compact icon-only presentation and remain excluded from close-family operations. In addition, the visible pin icon MUST act as a direct toggle affordance so the user can unpin the tab without opening the context menu.

#### Scenario: Pin a tab from the context menu

- **WHEN** the user right-clicks a tab and selects "Pin"
- **THEN** the tab moves to the left side of the tab bar and switches to compact presentation

#### Scenario: Unpin a tab from the context menu

- **WHEN** the user right-clicks a pinned tab and selects "Unpin"
- **THEN** the tab returns to normal size and moves back into the normal tab order

#### Scenario: Click the pin icon to unpin directly

- **WHEN** the user clicks the visible pin icon on a pinned tab
- **THEN** that tab is unpinned immediately
- **THEN** the icon hit target remains separate from tab-body activation
