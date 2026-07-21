## ADDED Requirements

### Requirement: Fullscreen scrolling preserves the selected image scale

The system MUST treat horizontal and vertical smooth scrolling over a fullscreen image as panning and MUST NOT change or reset the current zoom. Pinch or explicit zoom input MAY change zoom only when no smooth-scroll delta is being applied in the same interaction frame.

#### Scenario: Scroll a maximized image vertically

- **WHEN** the user enlarges an image and scrolls upward or downward over the fullscreen surface
- **THEN** the image moves vertically by the scroll delta
- **THEN** the selected zoom and rendered image dimensions remain unchanged

#### Scenario: Scroll a maximized image horizontally

- **WHEN** the user enlarges an image and scrolls left or right over the fullscreen surface
- **THEN** the image moves horizontally by the scroll delta
- **THEN** the selected zoom and rendered image dimensions remain unchanged

#### Scenario: Pinch zoom without scrolling

- **WHEN** the fullscreen surface receives a zoom gesture with no smooth-scroll delta
- **THEN** the image zoom changes within the configured minimum and maximum

### Requirement: Image overlay controls remain visible on every theme and image

The system MUST render the maximize control, lower-right pan/zoom/reset/info controls, and fullscreen close control with a fixed non-transparent dark background, a visible one-pixel light border, and light icons. Their base colors MUST NOT be derived from the active application theme or the pixels beneath the control.

#### Scenario: Show controls on a white image in a light theme

- **WHEN** a white or transparent image is displayed while a light theme is active
- **THEN** every image overlay control has a fixed dark background
- **THEN** every control boundary is visible through its light border
- **THEN** every control icon remains distinguishable from the image

#### Scenario: Show controls on a dark image in a dark theme

- **WHEN** a dark image is displayed while a dark theme is active
- **THEN** the light icon and border remain visible
- **THEN** hover, active, and focus states change control alpha without adopting image or theme colors
