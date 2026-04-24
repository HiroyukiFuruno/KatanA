## ADDED Requirements

### Requirement: User can attach an image file from the command palette

The system SHALL expose image file attach as a command palette action when an editable Markdown document is active.

#### Scenario: Image attach command is discoverable

- **WHEN** the user opens the command palette while an editable Markdown document is active
- **THEN** the system shows an image attach command when the query matches image attach wording
- **THEN** selecting that result starts the image file attach flow

#### Scenario: Image attach command is unavailable without editable Markdown

- **WHEN** no editable Markdown document is active
- **THEN** the system MUST NOT present image attach as an executable command palette action

### Requirement: User can attach an image file from the editor controls

The system SHALL expose image file attach as an editor toolbar control when an editable Markdown document is active.

#### Scenario: Image toolbar control opens the OS file picker

- **WHEN** the user clicks the image attach control in the editor toolbar
- **THEN** the system opens the operating system file picker filtered to image files
- **THEN** choosing an image saves it through the configured image ingest path
- **THEN** the system inserts a relative Markdown image reference at the editor insertion point

#### Scenario: Image attach is available from the grouped editor context menu

- **WHEN** the user opens the editor context menu in an editable Markdown document
- **THEN** image ingest actions are available under a grouped image or ingest submenu
- **THEN** choosing the image file action opens the operating system file picker filtered to image files
- **THEN** choosing the clipboard image action uses the configured clipboard image ingest path

### Requirement: User can paste clipboard images with normal paste

The system SHALL support normal paste for clipboard image data in an editable Markdown document.

#### Scenario: Clipboard image normal paste

- **WHEN** the user focuses an editable Markdown document and the clipboard contains image data
- **THEN** the user can invoke the platform normal paste gesture such as `Command+V` on macOS
- **THEN** the system saves the image through the configured image ingest path
- **THEN** the system inserts a relative Markdown image reference at the editor insertion point

#### Scenario: Clipboard image file normal paste

- **WHEN** the user focuses an editable Markdown document and the clipboard contains a copied image file from the operating system file manager
- **THEN** the user can invoke the platform normal paste gesture
- **THEN** the system reads the copied image file
- **THEN** the system saves the image through the configured image ingest path
- **THEN** the system inserts a relative Markdown image reference at the editor insertion point

#### Scenario: Text paste remains normal text paste

- **WHEN** the user focuses an editable Markdown document and the clipboard contains text
- **THEN** the platform normal paste gesture inserts text into the Markdown buffer
- **THEN** the system MUST NOT replace the text paste with image ingest

### Requirement: Image ingest uses document-relative asset output

The system SHALL save pasted or attached images using the configured document-relative image save directory and insert a Markdown image reference that resolves from the active document.

#### Scenario: Asset directory is created and referenced

- **WHEN** the user attaches or pastes an image into an editable saved Markdown document
- **THEN** the system resolves the output directory relative to the active Markdown document
- **THEN** the system creates the directory when the image ingest settings allow it
- **THEN** the system inserts Markdown image syntax pointing to the saved asset with a relative path

#### Scenario: Unsaved or non-file document cannot ingest image

- **WHEN** the active document has no file path that can anchor a relative asset directory
- **THEN** the system MUST NOT write image bytes to an arbitrary location
- **THEN** the system reports or exposes an unavailable state for image ingest
