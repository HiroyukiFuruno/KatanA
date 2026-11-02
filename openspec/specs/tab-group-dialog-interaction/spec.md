## Purpose

Define reliable and unambiguous tab group interactions.

## Requirements

### Requirement: Group popup accepts real application input

The application SHALL accept group-name text through the production menu, focus, foreground and panel interaction paths.

#### Scenario: Enter a group name in a later frame

- **WHEN** the user opens group creation from a real tab menu and types in a later frame
- **THEN** the field is enabled, receives focus and reflects the entered text without activating background shortcuts

### Requirement: Successful group creation closes its subwindow

The application SHALL permit named and empty-name creation when the name and color pair is unique, and close the subwindow after success.

#### Scenario: Named or empty Enter creation

- **WHEN** the user presses Enter with a name or an empty field and the name and color pair is unique
- **THEN** exactly one group exists in addition to the pre-menu count and the subwindow remains closed on subsequent frames

### Requirement: Group cancellation discards unconfirmed edits

The application SHALL close the subwindow with Cancel or Escape, discard unconfirmed creation or edit drafts, and preserve existing groups and membership.

#### Scenario: Cancel an existing rename or a new group's name entry

- **WHEN** the user clicks Cancel or presses Escape
- **THEN** the subwindow closes, existing state is preserved, and an unconfirmed new group is not created

### Requirement: Group names are optional and name and color pairs are unique

The application SHALL reject only an exact name and color pair shared with another group in the same workspace. The same name with a different color, a different name with the same color, and an empty name with a different color SHALL be permitted. Creation, later renaming, later recoloring, and direct actions SHALL use the same validation and exclude the edited group itself.

Whitespace-only names SHALL compare as empty names so visually unnamed groups follow the same rule. Other names SHALL retain their exact case, Unicode, and whitespace. Colors SHALL compare by their RGBA value. Comparison SHALL NOT rewrite stored names or colors.

#### Scenario: Reject duplicate creation or editing atomically

- **WHEN** creation, rename, or recolor would duplicate another group's name and color, including two empty names of the same color
- **THEN** no group, membership, or persisted state changes and the editor remains available with a localized error

#### Scenario: Preserve legacy data and unchanged edits

- **WHEN** saved data contains duplicate groups or an existing group is confirmed without changes
- **THEN** loading preserves the saved groups and an unchanged edit can close without writing a new duplicate

### Requirement: Destination menus display each group's selected color

Both tab and Explorer group destination menus SHALL display the selected group color for named and unnamed groups, with the whole row selecting the correct group.

#### Scenario: Identify unnamed destination groups

- **WHEN** multiple unnamed groups with distinct colors are listed
- **THEN** each row displays its stored color and selecting its swatch or label targets that group's identifier

### Requirement: Group regression verification uses the full application

Automated full-application input and isolated distribution-binary operations SHALL verify the regression; standalone header rendering SHALL NOT substitute for them.

#### Scenario: Verify the released group interaction

- **WHEN** the candidate and published binary are tested with an isolated profile
- **THEN** text input, named/empty Enter, Cancel, Escape, persistent results and absence of unintended duplicate creation are recorded
