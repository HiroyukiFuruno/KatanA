## ADDED Requirements

### Requirement: Local image requests remain stable and do not block the interface

KatanA SHALL keep read/decode waiting off the UI caller, retain the ready texture while the source is unchanged, and distinguish pending, missing source, and read/decode failure.

#### Scenario: Open and revisit an unchanged original PNG

- **WHEN** the original approximately 100KB PNG is selected, rendered repeatedly, and revisited after another tab
- **THEN** phase timings and the exact input identity are recorded, and the ready texture is not discarded merely because another frame or tab selection occurs

### Requirement: Image controls occupy a separate lower region

KatanA SHALL place image control buttons below the image region in normal and fullscreen presentation and keep them disjoint from the fullscreen button even for short images.

#### Scenario: Short image with fullscreen and controls

- **WHEN** an image shorter than a control button is rendered
- **THEN** the allocated image/fullscreen region and lower control region do not overlap, and buttons remain operable
