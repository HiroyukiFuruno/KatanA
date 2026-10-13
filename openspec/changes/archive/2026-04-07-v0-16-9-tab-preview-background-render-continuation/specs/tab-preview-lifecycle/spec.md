## ADDED Requirements

### Requirement: Tab-scoped preview work continues across tab switches

The system MUST allow tab-scoped preview work to continue after the user switches to another tab, instead of canceling that work solely because the tab lost focus.

#### Scenario: Switch away while a diagram render is still pending

- **WHEN** the active tab contains a Mermaid, PlantUML, or Draw.io section that has not finished rendering and the user switches to a different tab
- **THEN** the original tab's preview work continues in the background
- **THEN** the original tab does not require a restart from the initial loading state when the user returns

#### Scenario: Switch away while a tab-owned image-backed preview section is still loading

- **WHEN** the active tab contains a tab-owned image-backed preview section, including an extracted local-image section, that has not finished loading and the user switches to a different tab
- **THEN** that load is not canceled solely because another tab became active
- **THEN** the result remains associated with the original tab

### Requirement: Per-tab preview state distinguishes loaded results from drawn results

The system MUST maintain per-tab preview lifecycle state that distinguishes a completed background result from a result that has already been attached and drawn in the visible preview.

#### Scenario: Background work completes while the tab is inactive

- **WHEN** a preview section finishes loading or rendering while its tab is inactive
- **THEN** the tab records that section as loaded even if it is not yet drawn in the visible preview
- **THEN** the next activation of that tab reuses the completed result instead of restarting the work

#### Scenario: Loaded but not yet drawn work becomes visible

- **WHEN** a tab becomes active while one of its preview sections is already loaded but not yet drawn
- **THEN** the preview attaches and draws the completed result
- **THEN** the section transitions out of its hydration-pending state without rerunning the background job

#### Scenario: Completion is observed only after tab reactivation

- **WHEN** a background completion was queued while the tab was inactive and is first observed on the next activation of that tab
- **THEN** the system still treats that section as a loaded result for the current valid generation
- **THEN** the preview hydrates it without restarting the original work

### Requirement: Section identity is stable within a source generation

The system MUST associate tab-owned preview results with a stable section identity within the current source generation so completions can be attached to the correct section.

#### Scenario: Two sections share the same diagram kind or similar content

- **WHEN** multiple tab-owned preview sections in the same document generation have similar or identical payload text
- **THEN** completions are matched by stable section identity within that generation rather than by payload text alone
- **THEN** each completion attaches to the intended section

### Requirement: Obsolete preview work is rejected on invalidation

The system MUST reject background preview completions that belong to an older source generation after the tab content has changed or the tab has been closed.

#### Scenario: Source changes while old background work is still running

- **WHEN** the user edits the document or explicitly refreshes the preview after a background job has already started
- **THEN** any completion from the older source generation is ignored
- **THEN** only work associated with the latest source generation may be drawn

#### Scenario: Tab closes before background work completes

- **WHEN** a tab is closed before one of its preview jobs completes
- **THEN** the system discards the result rather than attaching it to any remaining tab
