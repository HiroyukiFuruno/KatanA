## ADDED Requirements

### Requirement: Group rename completes without trapping tab input

KatanA SHALL open the initial group rename popup once and release it on confirm, Enter, Escape or outside click. The completion action SHALL be scoped to its group identity.

#### Scenario: Confirm a newly created group name

- **WHEN** a new group name is entered and confirmed with Enter
- **THEN** the name is retained in group state, the matching inline rename is cleared, and another tab/group accepts input

#### Scenario: Cancel and continue tab operations

- **WHEN** Escape or outside click cancels the popup
- **THEN** the popup does not reopen every frame, subsequent group/tab operations work, and another group's rename state is preserved
