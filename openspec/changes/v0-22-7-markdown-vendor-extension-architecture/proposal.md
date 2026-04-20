# KatanA マークダウン vendor 拡張アーキテクチャ

## 概要

`vendor/egui_commonmark_upstream/` はすでに subtree 化されているが、KatanA 固有の描画・操作・検索・アンカー収集ロジックが subtree の内部へ蓄積している。結果として upstream 更新のたびに `CommonMarkViewer` と `pulldown.rs` 周辺で差分が膨らみ、局所修正が別の描画経路を壊しやすい。

本変更では、KatanA 固有ロジックの本体を `katana-ui` の型付きモジュール群へ戻し、vendor 側の独自差分を「安定した拡張ブリッジ」と「upstream へ返せる汎用修正」に限定するための詳細設計と実装計画を定義する。subtree 同期そのものは vendor 運用手順として別管理し、製品固有の変更と混同しない。

## 問題

- upstream 参照先は [lampsitter/egui_commonmark](https://github.com/lampsitter/egui_commonmark/tree/master) で、2026-04-18 時点の `master` は commit `9cc31bd725bc417fc9980375357c18bdf7feee37` (`2026-03-26`, `Release 0.23`) だった
- 現在の subtree 差分は `egui_commonmark/src` で 3 ファイル、`egui_commonmark_backend/src` で 7 ファイルに集中している
- 最大の乖離は `vendor/egui_commonmark_upstream/egui_commonmark/src/parsers/pulldown.rs` で、現在 3,208 行あり、タスクリスト、絵文字、検索、見出しアンカー、引用ブロック、コードブロック、テーブル、`<details>` HTML など複数責務が混在している
- `CommonMarkViewer` 自体も単なるビューア構築の薄い層から、コールバックと状態を集約する機能集積点へ拡張されている
- すでに `render_table_fn` と `crates/katana-ui/src/preview_pane/extension_table.rs` という良い方向の先行事例があるが、設計原則として統一されていない

## 目的

1. vendor 更新時の競合面を最小化する
2. KatanA 固有ロジックを `katana-ui` 側へ戻し、lint / テスト / カバレッジの主戦場を製品コード側へ移す
3. 標準 upstream 描画経路と KatanA 拡張描画経路の切り替えを容易にする
4. `pulldown.rs` 修正時の波及範囲を、責務単位のモジュールとテストで閉じ込める

## 非目的

- subtree 自体を廃止すること
- upstream `egui_commonmark` のパーサ / レンダラを全面的に再実装すること
- `katana-core` の Markdown エクスポート経路をこの変更で作り替えること
- upstream へ返せる汎用修正まで KatanA 側モジュールへ無理に追い出すこと

## 提案方針

1. `katana-ui` に `markdown_viewer/` 系の新しい責務境界を作り、現在プレビュー呼び出し側へ散らばっている hook 構成をファサードへ集約する
2. vendor 側には一度だけ安定した拡張ブリッジを導入し、将来の KatanA 機能追加が `CommonMarkViewer` の field 追加や `pulldown.rs` の分岐追加を要求しない形にする
3. 現在のコールバック群 (`render_table_fn`, `custom_task_box_fn`, `custom_emoji_fn`, など) はブリッジへ収束させ、KatanA 実装本体はホスト配下の機能モジュールへ移す
4. subtree 同期手順、差分監査手順、許可一覧を文書化し、vendor 運用と製品実装を切り分ける

## 受け入れ条件

- vendor 側の独自差分は「拡張ブリッジ」と「upstream へ返せる汎用修正」に分類でき、KatanA 固有機能差分が vendor 側へ新規追加されない
- `katana-ui` に Markdown レンダラーファサード、モード切り替え、ホスト、機能モジュール、テストの階層が導入される
- 実装後、標準描画モードと KatanA 拡張描画モードを同一 API で切り替えられる
- テーブル / タスクリスト / 絵文字 / リスト強調 / アンカー / 検索 / コードブロック外観の責務がモジュール単位で分離される
- upstream 同期手順、差分監査手順、許可一覧が文書化される
