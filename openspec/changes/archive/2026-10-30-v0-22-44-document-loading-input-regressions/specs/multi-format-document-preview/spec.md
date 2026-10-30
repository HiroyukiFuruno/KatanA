## ADDED Requirements

### Requirement: Loading presentation SHALL be centered in the document viewport

The host SHALL use one shared loading presentation for supported document types and place its visible indicator and label at the horizontal and vertical center of the available document viewport, including nested centered layouts and bounded inline surfaces.

#### Scenario: PPTX is loading in a centered preview surface

- **WHEN** the document preview has no rendered frame yet
- **THEN** the shared indicator rectangle is centered on both axes of the preview area
- **AND** its localized label contains no unresolved placeholder

### Requirement: Active document paging SHALL respect input ownership

The host SHALL allow unmodified left and right arrows to page an activated PDF or PPTX without an undocumented extra click, while respecting text entry, search, modal dialogs, modified shortcuts, and page boundaries.

#### Scenario: User selects a paginated document before pressing an arrow

- **WHEN** a ready PDF or PPTX becomes the active preview and no later text or modal interaction owns the input
- **THEN** an unmodified right arrow requests exactly one next page when a next page exists
- **AND** tab shortcuts and text editing are not triggered by that same input

### Requirement: Pending document conversion SHALL NOT block independent image presentation

The host SHALL keep Explorer selection and local image read/decode/paint responsive while Office or HTML work is pending. The host SHALL NOT require an unrelated document worker to finish before a local image can be painted.

#### Scenario: Image is selected while Office or HTML remains pending

- **WHEN** a small valid local PNG is selected while other document work has not produced a frame
- **THEN** its production request path can progress to a painted texture without waiting for that unrelated work to finish
- **AND** unchanged repeated frames do not reset the image decode or texture
