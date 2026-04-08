## ADDED Requirements

### Requirement: Preset Management for Icon Configurations
The system SHALL allow users to define, save, load, and revert generic icon configuration presets.

#### Scenario: User saves a new preset
- **WHEN** the user selects "Save Preset As" and provides a unique name (e.g., "My Custom Icons")
- **THEN** the system persists the current icon overrides under this new preset name.

#### Scenario: User reverts to default preset
- **WHEN** the user selects "Revert to Default"
- **THEN** the system unloads all current icon color and vendor overrides, restoring the default monochrome/theme-based drawing behavior.

### Requirement: Vendor-based Default Icon Coloring
The system SHALL display standard icon themes (e.g. Feather, Heroicons, Lucide) using assigned vendor-specific default colors, while keeping Katana icons monochrome.

#### Scenario: Icon is rendered without override
- **WHEN** an icon from `feather` is rendered
- **THEN** it uses the vendor's default color (e.g., blueish hue) instead of matching the text color.
- **WHEN** an icon from `katana` is rendered
- **THEN** it uses `currentColor` (theme's text color).

### Requirement: UI Settings Icon Grouping
The system SHALL display icon previews grouped by their vendor (e.g., Katana, Feather, Heroicons, Lucide) within the settings panel.

#### Scenario: User opens the icon preview settings
- **WHEN** the user navigates to the Icon settings tab
- **THEN** icons are grouped into categories indicating their origin (e.g., "Katana", "Feather"), each potentially in a collapsible group or distinct section.

### Requirement: Advanced Icon Overrides
The system SHALL allow users to override the default vendor and color for any supported UI icon.

#### Scenario: User configures a custom color for a specific icon
- **WHEN** the user opens the advanced settings for the "Explorer" icon
- **THEN** they can pick a specific Hex color which overrides the vendor default.

#### Scenario: User changes the vendor for a specific icon
- **WHEN** the user opens the advanced settings for the "Copy" icon
- **THEN** they can select a different SVG from alternative vendors (if mapped/available), bypassing the default `define_icons!` assignment.
