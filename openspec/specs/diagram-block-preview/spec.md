## Purpose

This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.
## Requirements
### Requirement: Supported diagram block payloads are explicitly constrained

システムは、MVP のプレビュー経路で扱う図形描画 payload を次の形式に限定しなければならない（SHALL）。`mermaid` フェンス内の生 Mermaid source、`@startuml` と `@enduml` を含む `plantuml` フェンス内の生 PlantUML source、`<mxfile>` または `<mxGraphModel>` を含む `drawio` フェンス内の非圧縮 Draw.io XML を扱う。各フェンスはバッククォート（backtick）の ````` とチルダ（tilde）の `~~~` の両方を受け入れなければならない（SHALL）。

#### Scenario: Accept a supported Mermaid payload with backticks

- **WHEN** the active Markdown document contains a fenced `mermaid` block opened and closed with `````
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

- **WHEN** the active Markdown document contains a valid ````mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render a Mermaid flowchart with tildes

- **WHEN** the active Markdown document contains a valid `~~~mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

#### Scenario: Render a ZenUML payload through Mermaid renderer

- **WHEN** the active Markdown document contains a `mermaid` fenced block whose source starts with `zenuml`
- **THEN** the block is treated as a Mermaid diagram payload
- **THEN** the preview passes the source to the renderer instead of preserving it as raw Markdown

### Requirement: Preview image backgrounds follow the current theme

システムは、Markdown に埋め込まれた画像とダイアグラム画像を、固定の黒背景または白背景ではなく、現在の preview 背景色の上に表示しなければならない（SHALL）。

#### Scenario: Render an embedded PNG on a light theme preview background

- **WHEN** the active Markdown document contains an embedded PNG image
- **AND** the active theme uses a light preview background
- **THEN** the preview paints the image container with the active preview background
- **THEN** transparent image pixels are composited onto that preview background instead of black

#### Scenario: Render ZenUML on the active preview background

- **WHEN** the active Markdown document contains a ZenUML payload inside a `mermaid` fenced block
- **THEN** the preview passes the active preview background through the diagram theme snapshot
- **THEN** transparent diagram pixels are composited onto that preview background instead of fixed white

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

### Requirement: Diagram rendering failures do not collapse Markdown preview

The system MUST preserve the preview workflow when a supported diagram block cannot be rendered.

#### Scenario: Fail gracefully on an invalid or unsupported diagram payload

- **WHEN** a supported diagram block cannot be rendered successfully
- **THEN** the preview remains available for the rest of the Markdown document
- **THEN** the failing block is replaced with a clear fallback state that exposes the source and error context

### Requirement: ダイアグラムプレビューは現在のテーマスナップショットを使用する

システムは、アプリ起動時点のスナップショットや dark/light 切り替えだけに依存するのではなく、現在のテーマスナップショットに基づいてダイアグラムプレビューを描画しなければならない（SHALL）。kdr backed renderer を利用する Mermaid / Draw.io でも、KatanA が渡したテーマスナップショットを kcf の実描画が使用しなければならない（MUST）。

#### Scenario: Mermaid プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** Mermaid 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: PlantUML プレビューが同一モード内の色変更へ追従する

- **WHEN** ユーザーが dark/light モードを変えずに preview text color や関連テーマ色を変更した時
- **THEN** PlantUML 描画は更新後のテーマスナップショットを使用する
- **THEN** プレビューは旧色セットで描かれた古いダイアグラム画像を再利用しない

#### Scenario: kdr backed Mermaid プレビューが light テーマを使用する

- **WHEN** KatanA の active theme が light mode の状態で Mermaid block を描画する
- **THEN** KatanA は kdr の `RenderInput` に light テーマの名前、背景、文字色、塗り、線、矢印、Mermaid theme を渡す
- **THEN** kdr が返す SVG は kdr 内部の dark 既定値ではなく、KatanA が渡した light テーマに基づく
- **THEN** 画面上の Mermaid 図形は dark 背景・白文字寄りの配色へ戻らない

#### Scenario: kdr backed Draw.io プレビューが light テーマを使用する

- **WHEN** KatanA の active theme が light mode の状態で Draw.io block を描画する
- **THEN** KatanA は kdr の `RenderInput` に light テーマの名前、背景、文字色、塗り、線、矢印、Draw.io label color を渡す
- **THEN** kdr が返す SVG は kdr 内部の dark 既定値ではなく、KatanA が渡した light テーマに基づく
- **THEN** 画面上の Draw.io 図形は dark 背景・白文字寄りの配色へ戻らない

### Requirement: ダイアグラムキャッシュキーはテーマ差分を識別する

システムは、永続化されるダイアグラムキャッシュキーに active なダイアグラムテーマの fingerprint を含めなければならない（SHALL）。kdr backed renderer では、KatanA 側の cache key と kdr 側の `cache_fingerprint` が、実描画に使われたテーマ差分で変化しなければならない（MUST）。

#### Scenario: テーマ fingerprint が変化する

- **WHEN** 同じ markdown file、diagram kind、source に対して active なダイアグラムテーマ fingerprint が変わった時
- **THEN** キャッシュキーは変化する
- **THEN** システムは古いキャッシュ結果を再利用せず、ダイアグラムを再描画する

#### Scenario: kdr runtime と profile の差分で cache key が変化する

- **WHEN** kdr の runtime version または renderer profile が変わった時
- **THEN** KatanA の diagram cache key は変化する
- **THEN** KatanA は古い kdr 出力を再利用しない

#### Scenario: kdr の crate version を手書きで固定しない

- **WHEN** KatanA が kdr backed renderer の cache key または backend version を組み立てる
- **THEN** system は実際の `katana-diagram-renderer` dependency version、`RenderOutput.runtime`、`RenderOutput.profile` から識別情報を得る
- **THEN** `crate=katana-diagram-renderer:0.1.0` のような古い手書き文字列を cache invalidation の根拠にしない

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

### Requirement: V8 を使う図形プレビュー依存関係はバージョン整合している

システムは、Mermaid / Draw.io プレビュー（preview）で利用する V8 を使う描画依存関係（V8-backed renderer dependencies）を、作業領域（workspace）内とユーザーレビュー用の `scripts/screenshot` manifest 内で単一の互換 `v8` バージョンに揃えなければならない（MUST）。同じプロセス内の数式描画（MathJax）経路は V8 を初期化してはならない（MUST NOT）。対応済み図形ブロック（diagram block）は、`katana-canvas-forge`、`katana-diagram-renderer`、または数式描画依存の不整合によりワーカー（worker）起動前に失敗してはならない（MUST NOT）。

#### Scenario: 作業領域の依存関係が同じ V8 固定指定を使う

- **WHEN** KatanA v0.22.19 向けに作業領域の依存関係（workspace dependencies）を解決する
- **THEN** `katana-canvas-forge` は `0.1.7` として解決される
- **THEN** 作業領域の `v8` は `=147.4.0` として解決される
- **THEN** `katana-canvas-forge` と `katana-diagram-renderer` は競合する `v8` バージョンを要求しない
- **THEN** 数式描画依存は `v8` を要求しない
- **THEN** `scripts/screenshot` manifest は非 V8 `mathjax_svg` patch を使う

#### Scenario: Mermaid プレビューのワーカーは描画前に切断されない

- **WHEN** 開いている Markdown 文書に対応済み Mermaid ブロックが含まれる
- **THEN** プレビューは V8 を使う描画ワーカーをバージョン競合による panic なしで起動する
- **THEN** 描画を試みる前に、ブロックが `[Mermaid] Diagram render worker disconnected before producing a result.` へ置換されない

#### Scenario: Draw.io プレビューは整合した実行環境を使う

- **WHEN** 開いている Markdown 文書に対応済み Draw.io ブロックが含まれる
- **THEN** プレビューは Mermaid 描画と同じ、作業領域で整合した V8 実行環境（runtime）を使う
- **THEN** kcf と kdr の `v8` バージョン分裂によりブロックが失敗しない

