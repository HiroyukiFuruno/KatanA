# Delta Spec: Workspace Shell — Foreground Surface Input Isolation

## ADDED Requirements

### Requirement: Foreground surfaces block pointer interaction from reaching background panes

The system MUST prevent hover, click, and context-menu interaction from reaching workspace-tree, editor, and preview background panes while any foreground window, popup, menu, overlay, or detached preview surface is active. This includes at minimum the settings window, command palette, file/markdown search modal, terms/about/meta/update/file-operation dialogs, tab/workspace context menus, activity-rail history menus, workspace breadcrumb menus, tab-group popups, settings-local popups such as the font selector and save-theme/no-extension dialogs, the splash overlay, and fullscreen/slideshow overlays.

#### Scenario: Pointer moves across preview while the settings window is open

- **WHEN** the user moves the pointer across the preview area while the settings window remains open
- **THEN** preview hover highlight does not update
- **THEN** editor-side current/hover indication does not change as a result

#### Scenario: Command palette is open above the workspace

- **WHEN** the user moves the pointer or clicks in the workspace tree, editor, or preview while the command palette remains open
- **THEN** background selection, hover highlight, and context-menu state do not change
- **THEN** only the command palette receives pointer interaction

#### Scenario: File search modal is open above the workspace

- **WHEN** the user moves the pointer or clicks in the workspace tree, editor, or preview while the file/markdown search modal remains open
- **THEN** background selection, hover highlight, and context-menu state do not change
- **THEN** only the search modal receives pointer interaction

#### Scenario: Tab or workspace popup is open above the shell

- **WHEN** the user clicks or hovers the tab strip, workspace tree, editor, or preview background while a tab context menu, workspace tree context menu, history menu, breadcrumb menu, or tab-group popup is open
- **THEN** the active tab state and preview hover state behind the menu do not change
- **THEN** only the foreground menu receives pointer interaction

#### Scenario: Settings-local popup or dialog is open

- **WHEN** the user opens a settings-local foreground surface such as the font selector, save-theme dialog, or no-extension warning
- **THEN** background workspace-tree, editor, and preview interaction remains blocked
- **THEN** the foreground settings surface itself remains interactive

#### Scenario: Fullscreen, splash, or detached preview surface is active

- **WHEN** the user has opened a fullscreen image viewer, slideshow surface, splash overlay, or detached preview surface
- **THEN** background shell panes do not receive pointer interaction
- **THEN** close, zoom, pan, and other interaction on the foreground surface itself remain available
