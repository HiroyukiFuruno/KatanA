## Context

KatanAの現行Markdown機能は、preview、export、editor支援、HTML badge、diagram、math、emoji、scroll syncに分散している。特に表（table/grid）、ホバー強調、コード位置同期、badge横並び、脚注、alert、相対画像、絵文字は、既存parserやrendererの都合に寄せると再びhackになる。

KMEはMarkdown parserの置き換えではなく、KatanA ecosystemが共有する文書構造の正本として設計する。

## Goals

- KME v0の対象を、現在KatanAで実現できているMarkdown挙動の踏襲として固定する。
- `sample.md`、README badge、alert、description listを仕様fixtureにする。
- metadataをMarkdown本文に埋め込まず、外部ファイルで管理する。
- editor保存時にmetadataを更新できる責務分担を定義する。
- previewはFloemネイティブ表示を前提にし、egui継続を前提にしない。
- kcf v0.1.2をKME実装ではなく品質ゲート構築として切る。
- 分離順序を P0 `katana-ast-lint`、P1 `katana-markdown-engine`、P2 `katana-ui-widget`、P3 その他として固定する。
- `katana-ui-widget` 分離をKME後続の必須境界として扱う。

## Non-Goals

- KME v0でCommonMark完全準拠を宣言すること。
- Markdown本文へKatanA専用ページング記法やLLM注釈を埋め込むこと。
- previewをHTML/WebViewへ寄せること。
- egui実装を前提に残すこと。
- kcf v0.1.2でKME本体を実装すること。

## Decisions

### KME Repository Ownership

KMEは `katana-markdown-engine` repositoryとして作成する。KMEはkcf、kdp、KatanA、editorへ依存しない。依存方向は利用側からKMEへの一方向にする。

### Separation Priority

P0は `katana-ast-lint` とする。分離対象repositoryが増えるほど、AST lintの独自実装や例外設定がrepositoryごとに分岐しやすくなる。KMEより先に共通AST lintを切り出し、以後のrepository分離で同じ品質ゲートを使えるようにする。

P1は `katana-markdown-engine`、P2は `katana-ui-widget`、P3はkdp/kle/kcf/KatanA統合などの周辺接続とする。

### Fixture Contract

主fixtureは `assets/fixtures/sample.md` とする。README冒頭のbadge列は実運用fixtureとして別扱いで必須にする。alertは `sample_basic.md` を利用し、description listは不足fixtureとして追加する。

### Metadata Contract

metadata標準名は `README.md.metadata.json` とする。targetはfile path、node id、byte range、line-column、対象text fingerprint、前後文脈を持つ。KMEはtarget再対応、移動、衝突、unresolved判定を返す。

### Editor Responsibility

保存直後のmetadata更新は `katana-language-editor` が主責務を持つ。editorは編集前後の本文と差分を知っているため、KMEの位置解決APIを使ってmetadataを更新する。

### Preview Responsibility

`katana-document-preview` はKME文書モデルをFloemで高速に表示する。hover、選択、AST単位コピー、unresolved metadata表示の入口を持つ。

### Export Responsibility

`katana-canvas-forge` はKMEモデルと解決済みmetadataを使う高再現exportを後続で担う。v0.1.2では、KME実装に入らず、壊れたことを検知できるGUI品質ゲートを作る。

### UI Widget Boundary

`katana-ui-widget` はFloem前提の共通UI部品repositoryとしてKME後続に分離する。metadata badge、unresolved表示、tab、toolbar、copy/edit affordanceをKatanA本体へ閉じ込めない。既にkcuで見えている課題を踏まえ、KMEのmetadata/display DTOを待ってから分離範囲を確定する。

## Risks

- KMEを既存parserのASTラッパーにすると、表や同期位置の微調整が再びhackになる。
- metadataを行番号だけで持つと、保存後に注釈やページングがずれる。
- previewとexportの仕様が別れると、見た目と出力が乖離する。
- UI部品分離を遅らせると、KME統合時にKatanA本体のFloem部品が肥大化する。
- AST lint共通化を遅らせると、分離後repositoryごとにlint基準が割れ、品質統制が効かなくなる。
