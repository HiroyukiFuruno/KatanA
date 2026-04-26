## MODIFIED Requirements

### Requirement: Supported diagram block payloads are explicitly constrained

システムは、MVP のプレビュー経路で扱う図形描画 payload を次の形式に限定しなければならない（SHALL）。`mermaid` フェンス内の生 Mermaid source、`@startuml` と `@enduml` を含む `plantuml` フェンス内の生 PlantUML source、`<mxfile>` または `<mxGraphModel>` を含む `drawio` フェンス内の非圧縮 Draw.io XML を扱う。各フェンスはバッククォート（backtick）の ` ``` ` とチルダ（tilde）の `~~~` の両方を受け入れなければならない（SHALL）。

#### Scenario: Accept a supported Mermaid payload with backticks

- **WHEN** the active Markdown document contains a fenced `mermaid` block opened and closed with ` ``` `
- **THEN** the block is treated as a supported diagram payload

#### Scenario: Accept a supported Mermaid payload with tildes

- **WHEN** the active Markdown document contains a fenced `mermaid` block opened and closed with `~~~`
- **THEN** the block is treated as a supported diagram payload

#### Scenario: Accept a supported PlantUML payload with tildes

- **WHEN** the active Markdown document contains a fenced `plantuml` block opened and closed with `~~~`
- **THEN** the block is treated as a supported diagram payload
- **THEN** the PlantUML source still requires explicit `@startuml` and `@enduml` delimiters

#### Scenario: Accept a supported Draw.io payload with tildes

- **WHEN** the active Markdown document contains a fenced `drawio` block opened and closed with `~~~`
- **THEN** the block is treated as a supported diagram payload
- **THEN** the Draw.io source still requires raw uncompressed XML containing `<mxfile>` or `<mxGraphModel>`

#### Scenario: Reject unsupported diagram encodings

- **WHEN** a diagram block relies on compressed XML, base64 payloads, or external file references that are outside the MVP input contract
- **THEN** the block is handled as an unsupported payload and rendered through the diagram failure fallback path

### Requirement: Mermaid blocks render inline in the standard preview

システムは、`mermaid` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。

#### Scenario: Render a Mermaid flowchart with backticks

- **WHEN** the active Markdown document contains a valid ` ```mermaid ` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render a Mermaid flowchart with tildes

- **WHEN** the active Markdown document contains a valid `~~~mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

### Requirement: PlantUML blocks render inline in the standard preview

システムは、`plantuml` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。

#### Scenario: Render a PlantUML sequence diagram with tildes

- **WHEN** the active Markdown document contains a valid `~~~plantuml` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the rendered result is produced through a fully local bundled rendering path compatible with the desktop application

### Requirement: Draw.io blocks render inline in the standard preview

システムは、`drawio` とラベル付けされたバッククォートまたはチルダの Markdown フェンスを、標準プレビュー上でインライン図形として描画しなければならない（SHALL）。

#### Scenario: Render an embedded Draw.io diagram block with tildes

- **WHEN** the active Markdown document contains a valid `~~~drawio` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the diagram is rendered without requiring the user to install a separate viewer

### Requirement: Diagram fences do not leak from non-diagram code blocks

システムは、図形ではないコードフェンスの内側にある `mermaid` / `plantuml` / `drawio` 文字列を、図形描画ブロックとして誤抽出してはならない（MUST NOT）。

#### Scenario: Nested tilde Mermaid inside markdown fence remains code

- **WHEN** the active Markdown document contains an outer `~~~markdown` fence that includes an inner `~~~mermaid` example
- **THEN** the preview keeps the outer fenced block as Markdown code content
- **THEN** the inner `~~~mermaid` example is not extracted as a diagram section

### Requirement: Code block insertion uses enum-backed language selection

システムは、コードブロック生成時に、何のコードブロックかをプルダウンで選択できるようにしなければならない（SHALL）。プルダウンの選択肢と挿入される fence info string は、同じ enum から解決しなければならない（SHALL）。

#### Scenario: Insert a text code block from the language selector

- **WHEN** user opens the code block insertion UI
- **THEN** system shows a language selector backed by the code block kind enum
- **WHEN** user selects `text`
- **THEN** system inserts a fenced code block with `text` as the info string

#### Scenario: Insert a shell code block from the language selector

- **WHEN** user selects `bash` or `zsh` from the code block language selector
- **THEN** system inserts a fenced code block whose info string matches the selected shell language

#### Scenario: Insert a diagram code block from the language selector

- **WHEN** user selects `mermaid`, `drawio`, or `plantuml` from the code block language selector
- **THEN** system inserts a fenced code block whose info string matches the selected diagram language
- **THEN** the inserted block is eligible for the diagram preview behavior defined in this spec

#### Scenario: Offer common development languages

- **WHEN** user opens the code block language selector
- **THEN** system includes `text`, `markdown`, `bash`, `zsh`, `mermaid`, `drawio`, and `plantuml`
- **THEN** system also includes common development languages such as `json`, `yaml`, `toml`, `rust`, `typescript`, `javascript`, `python`, `html`, `css`, and `sql`
