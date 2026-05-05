## MODIFIED Requirements

### Requirement: Mermaid blocks render inline in the standard preview

システムは、`mermaid` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。この描画は外部 library `katana-canvas-forge`（kcf）の版付き Renderer interface を通して行い、KatanA は Mermaid.js の描画内部処理を直接所有してはならない（MUST NOT）。

#### Scenario: Render a Mermaid flowchart with backticks

- **WHEN** the active Markdown document contains a valid ````mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render a Mermaid flowchart with tildes

- **WHEN** the active Markdown document contains a valid `~~~mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render Mermaid through kcf

- **WHEN** the preview renders a Mermaid block
- **THEN** KatanA passes the source, Mermaid.js-compatible `RenderConfig`, KatanA `RenderPolicy`, and theme `RenderContext` to the kcf `Renderer`
- **THEN** the preview consumes the returned SVG and `RenderDiagnostics` without depending on an unversioned `mermaid.min.js` shipped inside KatanA

### Requirement: Draw.io blocks render inline in the standard preview

システムは、`drawio` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。Draw.io 描画も kcf を経由しなければならない（MUST）。

#### Scenario: Render an embedded Draw.io diagram block with tildes

- **WHEN** the active Markdown document contains a valid `~~~drawio` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the diagram is rendered without requiring the user to install a separate viewer

#### Scenario: Render Draw.io through kcf

- **WHEN** the preview renders a Draw.io block
- **THEN** KatanA passes the Draw.io source through the kcf `Renderer` (Mermaid と同じ trait か別 backend かは kcf 側の実装判断)
- **THEN** KatanA does not silently launch the user's OS Chrome / Chromium app as a replacement path
