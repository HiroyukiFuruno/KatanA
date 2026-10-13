## Context

KatanAの現行Markdown機能は、viewer、export、editor支援、HTML badge、diagram、math、emoji、scroll syncに分散している。特に表（table/grid）、ホバー強調、コード位置同期、badge横並び、脚注、alert、相対画像、絵文字は、既存parserやrendererの都合に寄せると再びhackになる。

KMMはMarkdown parserの置き換えではなく、KatanA ecosystemが共有する文書構造の正本として設計する。

## Current Review State

全体計画は、KMMを起点にした一方向依存として再整理する。

現時点で進めてよいことは、KALの共通品質ゲート利用、KMMの文書モデルとmetadata contractの初期実装、親OpenSpecと各repo OpenSpecのpending条件整理である。

現時点で進めてはいけないことは、KDV、KLE、KCF、KatanA統合、KUWがKMM未確定のdocument modelやmetadata schemaを独自に確定することである。

## Goals

- KMM v0の対象を、現在KatanAで実現できているMarkdown挙動の踏襲として固定する。
- `sample.md`、README badge、alert、description listを仕様fixtureにする。
- metadataをMarkdown本文に埋め込まず、外部ファイルで管理する。
- editor保存時にmetadataを更新できる責務分担を定義する。
- viewerはFloemネイティブ表示を前提にし、egui継続を前提にしない。
- KDVがviewerとHTML/PDF/PNG/JPG exportを担う。
- KRRはMermaid、Draw.io、PlantUML、mathなどの外部描画 runtime を担う。
- 分離順序を P0 `katana-ast-lint`、P1 `katana-markdown-model`、P2 `katana-ui-widget`、P3 その他として固定する。
- `katana-ui-widget` 分離をKMM後続の必須境界として扱う。

## Non-Goals

- KMM v0でCommonMark完全準拠を宣言すること。
- Markdown本文へKatanA専用ページング記法やLLM注釈を埋め込むこと。
- viewerをHTML/WebViewへ寄せること。
- egui実装を前提に残すこと。
- KCFでKMM本体や新規export責務を実装すること。
- KMM、KLE、KDVへeditor-viewer同期制御を持たせること。

## Decisions

### KMM Repository Ownership

KMMは `katana-markdown-model` repositoryとして作成する。KMMはKCF、KDV、KatanA、editorへ依存しない。依存方向は利用側からKMMへの一方向にする。

### Separation Priority

P0は `katana-ast-lint` とする。分離対象repositoryが増えるほど、AST lintの独自実装や例外設定がrepositoryごとに分岐しやすくなる。KMMより先に共通AST lintを切り出し、以後のrepository分離で同じ品質ゲートを使えるようにする。

P1は `katana-markdown-model`、P2は `katana-ui-widget`、P3はKDV/KLE/KCF/KatanA統合などの周辺接続とする。

### Fixture Contract

主fixtureは `assets/fixtures/sample.md` とする。README冒頭のbadge列は実運用fixtureとして別扱いで必須にする。alertは `sample_basic.md` を利用し、description listは不足fixtureとして追加する。

### Metadata Contract

metadata標準名は `README.md.metadata.json` とする。targetはfile path、node id、byte range、line-column、対象text fingerprint、前後文脈を持つ。KMMはtarget再対応、移動、衝突、unresolved判定を返す。

### Editor Responsibility

保存直後のmetadata更新は `katana-language-editor` が主責務を持つ。editorは編集前後の本文と差分を知っているため、KMMの位置解決APIを使ってmetadataを更新する。

### Viewer Responsibility

`katana-document-preview` は未リリース・未取り込みのため、`katana-document-viewer`（KDV）へ改名する。

KDVはKMM文書モデルをFloemで高速に表示する。hover、選択、AST単位コピー、unresolved metadata表示の入口を持つ。KDVはKMM public DTOを入力にしたHTML/PDF/PNG/JPG exportも担い、viewer表示とexportを同じrender pipelineへ寄せる。

### Export Responsibility

`katana-render-runtime` はMermaid、Draw.io、PlantUML、mathなどの外部描画 runtime を担う。

`katana-canvas-forge` はHTML/PDF/PNG/JPG exportを担う。KDV側に同等機能が入るまで維持する。KDV実装後、KCFのexport関連計画と実装はKDVへ移譲し、KCF側から削除する。

### Editor-Viewer Sync Responsibility

editor-viewer同期制御はKatanAが担う。KMMはnode id、source range、line-column、raw snippet、fingerprintを返すだけで、scroll state、viewport、hit-test方針、selection、highlightを知らない。

KatanAが命令する先はviewerまたはeditorであり、KMMへscrollやselectionの命令は送らない。

### UI Widget Boundary

`katana-ui-widget` はFloem前提の共通UI部品repositoryとしてKMM後続に分離する。metadata badge、unresolved表示、tab、toolbar、copy/edit affordanceをKatanA本体へ閉じ込めない。既にkcuで見えている課題を踏まえ、KMMのmetadata/display DTOを待ってから分離範囲を確定する。

### KCF Existing Export Boundary

KCFの新規export計画はKDVより先に進めない。

理由は、viewer/exportの同一pipelineをKDVが担う方針に変わったためである。KMM文書モデル、metadata schema、KUWの共通UI境界、viewer/editor責務が固まる前にKCF側で新規export仕様を固定しない。

KCFで許可する範囲は、Mermaid、Draw.io、PlantUML、mathなどの外部描画、既存CSS export回帰修正の保全、既存harness実装の保守、起動手順の記録、KMM/KUW/KDV/KLE確定後のOpenSpec更新だけである。

### Repository DoR / DoD

`katana-ast-lint`:

- DoR: repository分離後も共通AST lint gateを使う方針が親OpenSpecで確定している。
- DoD: KMMや後続repoが、repository-localな一時lintや除外設定ではなくKALを品質ゲートとして使える。

`katana-markdown-model`:

- DoR: KALが利用でき、KMM v0のfixture基準とmetadata方針が親OpenSpecで確定している。
- DoD: KatanA現行fixtureを文書モデル化でき、stable id、source range、raw snippet、fingerprint、metadata target解決、raw fallbackをKMM public DTOとして検証済みである。KCF、KDV、KatanA、editorへ依存しない。editor-viewer同期制御を持たない。

`katana-ui-widget`:

- DoR: KMMのmetadata/display DTOが利用可能で、KatanA/KDV/KLEで共有すべきFloem部品候補が列挙されている。
- DoD: metadata badge、unresolved表示、tabs、toolbar、copy/edit affordanceを共通UI部品として提供でき、KMM文書モデルやmetadata schemaを所有しない。

`katana-document-viewer`:

- DoR: KMM public DTOとKUW境界が利用可能である。
- DoD: KMM modelからFloem native viewerを表示し、hover、選択、AST単位copy、unresolved metadata表示の入口を持つ。HTML/PDF/PNG/JPG exportはviewerと同じrender pipelineを使う。KMM parserを再実装しない。

`katana-language-editor`:

- DoR: KMM metadata schemaとtarget resolution APIが利用可能である。
- DoD: 保存直後にold source、new source、metadataからtargetを更新できる。自動復元できないtargetは削除せずunresolvedとして保持する。

`katana-render-runtime`:

- DoR: KMM文書モデル、metadata schema、KUW境界、viewer/editor責務が安定している。
- DoD: Mermaid、Draw.io、PlantUML、mathなどの外部描画 runtime を担う。

`katana-canvas-forge`:

- DoR: KMM文書モデル、metadata schema、KUW境界、viewer/editor責務が安定している。
- DoD: HTML/PDF/PNG/JPG exportを担う。既存exportはKDV移譲まで維持し、KDV実装後に削除できる状態である。

`katana`:

- DoR: KMM、KDV、KLE、KRR、KCF、KUWのpublic contractが利用可能である。
- DoD: KatanA本体は各libのpublic DTOだけを使って統合し、parser internals、renderer internals、widget private stateを永続stateへ持たない。editor-viewer同期制御をKatanAが持ち、viewerまたはeditorへ命令する。

## Risks

- KMMを既存parserのASTラッパーにすると、表や同期位置の微調整が再びhackになる。
- metadataを行番号だけで持つと、保存後に注釈やページングがずれる。
- viewerとexportの仕様が別れると、見た目と出力が乖離する。
- UI部品分離を遅らせると、KMM統合時にKatanA本体のFloem部品が肥大化する。
- AST lint共通化を遅らせると、分離後repositoryごとにlint基準が割れ、品質統制が効かなくなる。
- KUW repositoryが未作成のままKDVやKatanA本体でUI部品を増やすと、後から切り出す時にmetadata表示やcopy/edit操作の責務が崩れる。
- KCFの新規export計画を先に完了扱いにすると、KMM、KDV、KUWの境界変更で評価対象がずれる。
