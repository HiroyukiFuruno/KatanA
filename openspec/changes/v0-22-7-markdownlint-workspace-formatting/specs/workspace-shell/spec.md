## MODIFIED Requirements

### Requirement: Workspace tree reflects the files within the active project

システムは、アクティブなワークスペースを、ファイルとディレクトリを反映する操作可能なプロジェクトツリーとして表示しなければならない（SHALL）。プロジェクトツリーは、ファイル選択だけでなく、Markdown フォーマット、ファイル作成、フォルダ作成の入口も提供しなければならない（SHALL）。

#### Scenario: Render project contents after workspace load

- **WHEN** a workspace root has been loaded successfully
- **THEN** the system shows directories and files from that workspace in the project tree
- **THEN** the active document selection can be changed from that tree

#### Scenario: Open a document from the project tree

- **WHEN** the user selects a Markdown document in the project tree
- **THEN** the system loads that document into the editor
- **THEN** the preview pipeline uses that document as the active source

#### Scenario: Format a Markdown file from the file context menu

- **WHEN** user opens the context menu on a valid Markdown file in the explorer
- **THEN** system shows `ファイルをフォーマットする`
- **THEN** selecting it formats that file through KML using the effective markdownlint config
- **THEN** system refreshes the editor buffer and diagnostics after the file is saved

#### Scenario: Hide file formatting for unsupported files

- **WHEN** user opens the context menu on a non-Markdown file
- **THEN** system does not show `ファイルをフォーマットする`

#### Scenario: Open empty-space explorer context menu

- **WHEN** user opens the context menu on empty space below the explorer tree
- **THEN** system shows `ワークスペース内の Markdown を一括フォーマット`
- **THEN** system shows `ファイルの新規作成`
- **THEN** system shows `フォルダの新規作成`

#### Scenario: Format all Markdown files in workspace

- **WHEN** user selects `ワークスペース内の Markdown を一括フォーマット`
- **THEN** system formats all supported Markdown files under the active workspace through KML
- **THEN** system excludes hidden infrastructure directories that are already excluded from explorer traversal
- **THEN** system reports formatted file count and failed file count

#### Scenario: Create a file from explorer empty space

- **WHEN** user selects `ファイルの新規作成` from the explorer empty-space context menu
- **THEN** system opens the existing file creation dialog with the workspace root as parent directory

#### Scenario: Create a folder from explorer empty space

- **WHEN** user selects `フォルダの新規作成` from the explorer empty-space context menu
- **THEN** system opens the existing folder creation dialog with the workspace root as parent directory

### Requirement: Explorer header exposes file and folder creation actions

システムは、エクスプローラーのフィルター操作の左に、ファイル追加とフォルダ追加のアイコンボタンを表示しなければならない（SHALL）。

#### Scenario: Show creation icons near filter

- **WHEN** an active workspace is loaded
- **THEN** system shows a file-add icon immediately to the left of the filter icon
- **THEN** system shows a folder-add icon immediately to the left of the file-add icon

#### Scenario: Create file from header icon

- **WHEN** user clicks the file-add icon in the explorer header
- **THEN** system opens the existing file creation dialog with the workspace root as parent directory

#### Scenario: Create folder from header icon

- **WHEN** user clicks the folder-add icon in the explorer header
- **THEN** system opens the existing folder creation dialog with the workspace root as parent directory

#### Scenario: Creation icons use registered icon packs

- **WHEN** the active icon pack is Feather, Heroicons, Lucide, Material Symbols, or Tabler Icons
- **THEN** system renders native file-add and folder-add icons from that icon pack
- **THEN** system does not reuse a copied SVG from another icon pack
