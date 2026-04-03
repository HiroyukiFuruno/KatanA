## Why

現在の editor は `egui::TextEdit::multiline` による素の Markdown 編集であり、見出しや装飾、表、画像参照を毎回手打ちする必要がある。local image preview の土台はある一方で、画像添付、クリップボード貼り付け、`./asset/img` への保存、参照先の探索導線がなく、authoring 体験が分断されている。

## What Changes

- Markdown を主編集面としたまま、見出し、装飾、リスト、表などの挿入支援を追加する
- editor / menu / shortcut から Markdown 記法を挿入できる authoring command を追加する
- 画像ファイル選択とクリップボード画像貼り付けの両方に対応する image attach workflow を追加する
- 画像の保存先 default を、active Markdown file を起点にした `./asset/img` にする
- 画像名は timestamp を default としつつ、設定で保存先や命名ダイアログ有無を変更できるようにする
- Markdown 内の local image reference を解決し、workspace 上で対象ディレクトリやファイルへ辿れる導線を追加する

## Capabilities

### New Capabilities

- `markdown-asset-ingest`: Markdown 文書に対して local image の添付、貼り付け、保存、参照先導線を提供する

### Modified Capabilities

- `markdown-authoring`: Markdown source-first の編集体験に挿入支援と authoring command を追加する

## Impact

- 主な影響範囲は `crates/katana-ui/src/views/panels/editor/ui.rs`、editor action / command 層、`crates/katana-ui/src/preview_pane/*`、`crates/katana-ui/src/views/panels/workspace/*`、`crates/katana-platform/src/settings/*`
- clipboard image 取得と local file 保存のために platform integration が増える可能性がある
- `v0.19.0` の command inventory と `v0.20.0` の shortcut customization を前提にする
