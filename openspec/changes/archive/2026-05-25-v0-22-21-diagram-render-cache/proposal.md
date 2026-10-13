## Why

KatanA の図形（diagram）プレビューは、Mermaid / Draw.io / PlantUML 等のコードブロックから SVG を生成して画面へ表示する。現状はタブ切り替えや既存タブ復元のたびに描画ロジック（render logic）が再実行され、ユーザー検証では cache が体感できるほど効いていない。

原因は、キャッシュ（cache）の単位が「ASTで切り出した図形コードブロック」になっておらず、保存先・ヒット判定・削除条件が分離されていないためである。KatanA v0.22.21 では、Markdown ファイルの絶対パスでキャッシュ領域を分け、図形コードブロック本文のチェックサム（checksum）単位で生成済み SVG を保存・再利用する。

## What Changes

- Markdown を AST 解析し、図形系コードブロックを列挙する。
- Markdown ファイルの絶対パスから安定ハッシュを生成し、文書ごとの cache 領域を分離する。
- 図形コードブロック本文と図形種別から content checksum を生成する。
- cache payload は JSON ではなく SVG ファイルそのものとして保存する。
- `manifest.json` / `cache.json` は導入しない。
- cache hit 時は `katana-diagram-renderer (kdr)` を呼ばず、保存済み SVG を使用する。
- ドキュメント更新時は、現在の AST に存在しない checksum の SVG を削除する。
- タブ移動・タブ切り替え・選択状態変更・viewport 操作では、図形 cache 判定と再描画を行わない。

## Capabilities

### New Capabilities

- `diagram-render-cache`: Markdown 絶対パスと図形コードブロック本文 checksum に基づき、生成済み SVG を永続 cache として保存・再利用する capability。

### Modified Capabilities

- なし

## Impact

- `crates/katana-ui`: プレビュー更新時に AST から図形コードブロックを列挙し、SVG cache store を呼び出す統合点
- `crates/katana-platform`（または局所 module）: OS 別 cache root と atomic write helper
- `crates/katana-core`: 既存の図形描画結果を SVG cache payload として扱うための接続点
- `crates/katana-ui/tests` / `crates/katana-platform/tests`: cache hit / miss / prune / corrupt / path separation の回帰確認
- 観測性（observability）: `diagram_cache_hit` / `diagram_cache_miss` / `diagram_cache_pruned` / `diagram_cache_corrupt_svg` / `diagram_cache_redraw_executed` / `diagram_cache_skipped_by_tab_switch` の metric / log 追加
