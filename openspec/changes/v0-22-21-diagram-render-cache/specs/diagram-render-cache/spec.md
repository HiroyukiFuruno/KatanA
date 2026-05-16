## ADDED Requirements

### Requirement: Content-based Diagram Checksum

システムは、図形（diagram）の content checksum を、正規化済みの図形内容のみから生成しなければならない（SHALL）。タブ状態、viewport 状態、選択状態、OS 名、cache 保存先、ファイルパス、タイムスタンプは checksum 計算に含めてはならない（MUST NOT）。

#### Scenario: Generate checksum from diagram content

- **GIVEN** a document contains one or more diagrams
- **WHEN** the system evaluates render cache validity
- **THEN** it SHALL canonicalize the diagram content
- **AND** it SHALL generate `contentChecksum` from the canonicalized content only
- **AND** it SHALL exclude tab state, viewport state, selection state, OS name, cache path, file path, and timestamps

#### Scenario: Same diagram content produces same checksum

- **GIVEN** two tabs reference the same document diagram content
- **AND** their tab IDs, viewport states, or selection states differ
- **WHEN** the system generates `contentChecksum`
- **THEN** both tabs SHALL produce the same checksum

#### Scenario: Changed diagram content produces different checksum

- **GIVEN** a diagram has been changed in a way that affects rendered output
- **WHEN** the system generates `contentChecksum`
- **THEN** the checksum SHALL differ from the previously cached checksum

### Requirement: Checksum-driven Redraw

システムは、現在の content checksum と cached content checksum が不一致の場合、または cache を安全に利用できない場合に限り、図形描画ロジックを実行しなければならない（SHALL）。

#### Scenario: Cache hit avoids redraw

- **GIVEN** a valid cache manifest exists for a document
- **AND** the manifest `contentChecksum` matches the current content checksum
- **AND** the cache schema version and renderer version are compatible
- **WHEN** the user opens a new tab for the document
- **THEN** the system SHALL load the rendered diagram payload from cache
- **AND** it SHALL NOT execute the diagram redraw logic

#### Scenario: Checksum mismatch triggers redraw

- **GIVEN** a valid cache manifest exists for a document
- **AND** the manifest `contentChecksum` does not match the current content checksum
- **WHEN** the system evaluates render cache validity
- **THEN** the system SHALL execute the diagram redraw logic
- **AND** it SHALL update the rendered payload and cache manifest

#### Scenario: Invalid cache triggers redraw fallback

- **GIVEN** the cache manifest or rendered payload is missing, corrupted, or incompatible
- **WHEN** the system attempts to use the render cache
- **THEN** the system SHALL treat the result as a cache miss
- **AND** it SHALL execute the diagram redraw logic
- **AND** it SHALL avoid surfacing the cache failure as a user-facing error

### Requirement: OS-specific Katana Cache Storage

システムは、図形描画 cache を OS ごとに解決される Katana app 用一時保存領域に保存しなければならない（SHALL）。アプリ側の描画ロジックは OS 固有のパスに直接分岐してはならない（MUST NOT）。

#### Scenario: Resolve platform cache area

- **GIVEN** Katana is running on macOS, Windows, or Linux
- **WHEN** the render cache store needs a persistence path
- **THEN** the system SHALL resolve the Katana app temporary cache area for the current OS
- **AND** application render logic SHALL NOT directly branch on OS-specific paths

#### Scenario: Store manifest and payload

- **GIVEN** diagram redraw logic has produced a rendered payload
- **WHEN** the system writes render cache data
- **THEN** it SHALL write a cache manifest containing `documentId`, `contentChecksum`, `cacheSchemaVersion`, `rendererVersion`, and `payloadPath`
- **AND** it SHALL write the rendered payload to the resolved Katana app temporary cache area
- **AND** it SHALL write manifest and payload atomically or recover safely from partial writes

#### Scenario: Load without full render load logic

- **GIVEN** a cache manifest and payload are valid
- **WHEN** the system needs to restore rendered diagrams
- **THEN** it SHALL hydrate the rendered diagram state from cache
- **AND** it SHALL avoid executing the normal diagram rendering load path

#### Scenario: Fallback when cache path is unavailable

- **GIVEN** the OS cache path cannot be resolved or cannot be written to
- **WHEN** the system attempts to persist render cache data
- **THEN** it SHALL fall back to an in-memory cache for the current session
- **AND** it SHALL NOT surface the failure as a user-facing error

### Requirement: Limited Checksum Evaluation Timing

システムは、図形 cache の checksum 判定を、新規タブ作成時、ドキュメント更新時、アプリ起動時の既存タブ復元時のみに実行しなければならない（SHALL）。タブ移動、タブ切替、選択状態のみの変更、viewport 操作のみを理由として checksum 判定や再描画を実行してはならない（MUST NOT）。

#### Scenario: Evaluate on new tab creation

- **GIVEN** a user opens a new tab for a document
- **WHEN** the tab is created
- **THEN** the system SHALL evaluate the current content checksum against the cached checksum

#### Scenario: Evaluate on document update

- **GIVEN** a document update is executed
- **WHEN** the updated diagram content is available
- **THEN** the system SHALL evaluate the current content checksum against the cached checksum
- **AND** it SHALL redraw only when the checksums differ

#### Scenario: Evaluate on startup restored tabs

- **GIVEN** Katana starts with tabs that were already open in the previous session
- **WHEN** the app restores those tabs
- **THEN** the system SHALL evaluate the current content checksum against the cached checksum for each restored tab

#### Scenario: Do not evaluate on tab movement

- **GIVEN** a user moves a tab or changes tab order
- **WHEN** the tab movement completes
- **THEN** the system SHALL NOT evaluate diagram cache checksum
- **AND** it SHALL NOT execute diagram redraw logic solely because of tab movement

#### Scenario: Do not evaluate on selection or viewport change

- **GIVEN** a user changes selection state, scroll position, or zoom level only
- **WHEN** the UI state change completes
- **THEN** the system SHALL NOT evaluate diagram cache checksum
- **AND** it SHALL NOT execute diagram redraw logic solely because of the UI state change

### Requirement: Diagram Cache Observability

システムは、cache 判定結果を診断（diagnostics）用に metric / log として記録しなければならない（SHALL）。metric 名は `diagram_cache_*` の snake_case で統一する。

#### Scenario: Record cache hit and miss

- **GIVEN** the system evaluates diagram render cache validity
- **WHEN** the evaluation completes
- **THEN** it SHALL record one of `diagram_cache_hit`, `diagram_cache_miss`, `diagram_cache_mismatch`, `diagram_cache_corrupt_payload`, or `diagram_cache_redraw_executed`

#### Scenario: Record checksum evaluation lifecycle

- **GIVEN** the system performs checksum evaluation on a supported event
- **WHEN** the evaluation completes
- **THEN** it SHALL record `diagram_cache_checksum_evaluated`

#### Scenario: Record skipped tab movement evaluation

- **GIVEN** a tab movement event occurs
- **WHEN** checksum evaluation is intentionally skipped
- **THEN** the system SHALL record `diagram_cache_checksum_skipped_by_tab_move`
