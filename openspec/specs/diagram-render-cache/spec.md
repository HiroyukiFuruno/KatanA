## Purpose

KatanA の Markdown preview は、Markdown AST から抽出した図形コードブロック単位で SVG cache を扱う。cache は OS 標準の一時保存領域に保存し、Markdown ファイルの絶対パスと図形内容に基づく checksum によって再利用可否を判定する。

## Requirements

### Requirement: AST Diagram Block Cache Unit

システムは、図形（diagram）cache の単位を Markdown AST から抽出した図形系コードブロックにしなければならない（SHALL）。

#### Scenario: Extract diagram code blocks from AST

- **GIVEN** a Markdown document contains Mermaid, Draw.io, or PlantUML code blocks
- **WHEN** the preview evaluates diagram cache entries
- **THEN** the system SHALL enumerate diagram code blocks from the Markdown AST
- **AND** it SHALL treat non-diagram code blocks as outside the persistent diagram cache

#### Scenario: Do not use block order as persistent key

- **GIVEN** a document contains multiple diagram code blocks
- **WHEN** a diagram code block is inserted, removed, or reordered
- **THEN** the system SHALL NOT use the AST order or source position as the persistent cache key
- **AND** it SHALL continue to identify reusable SVG files by diagram kind and content checksum

### Requirement: Document Path Separated SVG Storage

システムは、Markdown ファイルの絶対パスから生成した安定ハッシュごとに、SVG cache 保存領域を分離しなければならない（SHALL）。

#### Scenario: Resolve document-specific cache directory

- **GIVEN** a Markdown file has an absolute path
- **WHEN** the diagram cache store resolves a persistence directory
- **THEN** it SHALL generate a stable hash from that absolute path
- **AND** it SHALL store diagram SVG files under `${os_cache_dir}/KatanA/.cache/diagrams/doc_<absolute_path_hash>/`

#### Scenario: Same diagram content in different files is not shared

- **GIVEN** two Markdown files have different absolute paths
- **AND** both files contain the same diagram code block content
- **WHEN** the system stores diagram SVG cache entries
- **THEN** the files SHALL use different document cache directories
- **AND** the SVG files SHALL NOT be shared across those Markdown files

### Requirement: Content Checksum SVG File Naming

システムは、図形種別と図形コードブロック本文から content checksum を生成し、SVG ファイル名に含めなければならない（SHALL）。

#### Scenario: Generate checksum from diagram block content

- **GIVEN** a diagram code block has a kind and body
- **WHEN** the system evaluates cache validity
- **THEN** it SHALL generate `content_checksum` from the diagram kind and code block body
- **AND** it SHALL exclude tab state, viewport state, selection state, OS name, cache path, timestamps, AST order, and source position

#### Scenario: Cache hit reads SVG file

- **GIVEN** the expected `<content_checksum>_<renderer_version>_<theme_hash>.svg` file exists
- **AND** the SVG file can be read as valid cache payload
- **WHEN** the preview needs the diagram
- **THEN** the system SHALL use the cached SVG
- **AND** it SHALL NOT execute the diagram renderer for that diagram block

#### Scenario: Cache miss writes SVG file

- **GIVEN** the expected SVG cache file is missing or unreadable
- **WHEN** the preview needs the diagram
- **THEN** the system SHALL execute the diagram renderer for that diagram block
- **AND** it SHALL write the generated SVG file atomically

### Requirement: No Manifest Or JSON Payload

システムは、図形 SVG cache のヒット判定に `manifest.json` や `cache.json` を必須としてはならない（MUST NOT）。

#### Scenario: Store SVG as the cache payload

- **GIVEN** diagram redraw logic has produced SVG
- **WHEN** the system persists the cache payload
- **THEN** it SHALL store the SVG itself as the cache file
- **AND** it SHALL NOT require a separate JSON manifest to validate the cache hit

### Requirement: Prune Removed Diagram Blocks

システムは、ドキュメント更新時に現在の AST に存在しない図形 checksum の SVG を削除しなければならない（SHALL）。

#### Scenario: Remove deleted middle diagram

- **GIVEN** a Markdown file had seven diagram code blocks
- **AND** the middle diagram code block was deleted so the file now has six diagram code blocks
- **WHEN** the document update cache pass completes
- **THEN** the system SHALL keep SVG files whose checksums still exist in the current AST
- **AND** it SHALL delete SVG files whose checksums no longer exist in the current AST

#### Scenario: Reordered diagrams reuse cache

- **GIVEN** a Markdown file contains diagram code blocks A, B, and C
- **AND** the user reorders them to C, A, and B without changing their content
- **WHEN** the document update cache pass evaluates the diagrams
- **THEN** the system SHALL reuse the existing SVG files by checksum
- **AND** it SHALL NOT redraw solely because the diagram order changed

### Requirement: Limited Evaluation Timing

システムは、図形 cache 判定を新規タブ作成時、ドキュメント更新時、アプリ起動時の既存タブ復元時に限定しなければならない（SHALL）。

#### Scenario: Evaluate on supported events

- **GIVEN** a new tab opens, a document updates, or Katana restores existing tabs at startup
- **WHEN** the preview needs diagram content
- **THEN** the system SHALL evaluate the SVG cache for each current diagram code block

#### Scenario: Do not evaluate on tab-only events

- **GIVEN** a user switches tabs, moves tabs, changes selection, scrolls, or zooms only
- **WHEN** the UI state change completes
- **THEN** the system SHALL NOT evaluate diagram cache checksums
- **AND** it SHALL NOT execute diagram redraw logic solely because of that UI state change

### Requirement: Diagram Cache Observability

システムは、cache 判定結果を診断（diagnostics）用に metric / log として記録しなければならない（SHALL）。metric 名は `diagram_cache_*` の snake_case で統一する。

#### Scenario: Record cache lifecycle

- **GIVEN** the system evaluates diagram SVG cache validity
- **WHEN** the evaluation completes
- **THEN** it SHALL record cache hit, miss, prune, corrupt SVG, or redraw execution as `diagram_cache_*` diagnostics
