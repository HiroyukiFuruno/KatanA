# Delta Spec: Local Asset Preview — Tab-Switch Continuity

## ADDED Requirements

### Requirement: Extracted local-image preview loading survives tab switches

The system SHALL preserve tab ownership for extracted local-image preview loading across tab switches and must not restart that work solely because another tab became active.

#### Scenario: Switch tabs before a local image section is fully ready

- **WHEN** the user switches away from a tab before one of its extracted local-image preview sections is fully ready
- **THEN** the original tab retains the loading lifecycle for that local-image section
- **THEN** revisiting the tab does not force the section back to its initial loading state if the background result is still valid

### Requirement: Loaded image-backed results draw on the next activation

The system SHALL attach already loaded extracted local-image results to the visible preview when their tab becomes active again.

#### Scenario: Image-backed result became loaded while the tab was inactive

- **WHEN** an extracted local-image preview section reaches the loaded state while its tab is inactive
- **THEN** returning to that tab draws the already loaded result
- **THEN** the system does not require a duplicate load for the same valid asset

#### Scenario: Loaded but not yet drawn image-backed state is visible in the lifecycle

- **WHEN** a tab owns an extracted local-image preview result that is already loaded but not yet drawn
- **THEN** the preview lifecycle keeps that state distinct from fully drawn output
- **THEN** activation of the tab consumes that loaded result into the visible preview
