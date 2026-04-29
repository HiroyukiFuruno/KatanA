## MODIFIED Requirements

### Requirement: Mermaid blocks render inline in the standard preview

システムは、`mermaid` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。この描画は、版付き Mermaid runtime interface を通して行い、KatanA は Mermaid.js の描画内部処理を直接所有し続けてはならない（MUST NOT）。

#### Scenario: Render a Mermaid flowchart with backticks

- **WHEN** the active Markdown document contains a valid ````mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render a Mermaid flowchart with tildes

- **WHEN** the active Markdown document contains a valid `~~~mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render Mermaid through the versioned runtime interface

- **WHEN** the preview renders a Mermaid block
- **THEN** KatanA passes the source, Mermaid.js-compatible config, KatanA policy, and theme context to the Mermaid runtime interface
- **THEN** the preview consumes the returned SVG and diagnostics without depending on an unversioned `mermaid.min.js`

### Requirement: Draw.io blocks render inline in the standard preview

システムは、`drawio` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。Draw.io runtime の所有境界が Mermaid runtime と異なる場合、システムは OS アプリ依存へ戻さず、未接続または後続移管の扱いを明示しなければならない（MUST）。

#### Scenario: Render an embedded Draw.io diagram block with tildes

- **WHEN** the active Markdown document contains a valid `~~~drawio` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the diagram is rendered without requiring the user to install a separate viewer

#### Scenario: Keep Draw.io ownership explicit

- **WHEN** Draw.io rendering cannot use the Mermaid runtime interface
- **THEN** the system records the separate ownership boundary
- **THEN** the system does not silently launch the user's OS Chrome / Chromium app as a replacement path
