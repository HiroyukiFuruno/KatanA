## Why

0.22.7 では、Markdown 編集体験の中で分断している三つの不整合をまとめて直す必要がある。

一つ目は、図形描画ブロックがバッククォート（backtick）の ````` だけを前提としており、markdownlint の正式パターンとして使われるチルダ（tilde）の `~~~` で描かれた `mermaid` / `plantuml` / `drawio` がプレビューやエクスポートで図形として扱われないこと。

二つ目は、Lint 設定で「ワークスペースの `.markdownlint.json` を見る」を切り替えた時に、画面が意図せず高度な設定画面へ切り替わることと、切り替えても実際の診断へ反映されないこと。一般設定で扱う「無視 / 警告 / エラー」と、`.markdownlint.json` で扱うルール適用設定が混ざっており、どちらが正になるか分かりにくい。

三つ目は、KML（katana-markdown-linter）のフォーマット機能をアプリ内から自然に呼び出す導線がないこと。ファイル単位、ワークスペース単位のフォーマット操作に加え、エクスプローラー上でファイル/フォルダ作成をすぐ行える入口も不足している。

## What Changes

- ````mermaid` などのバッククォートフェンスに加え、`~~~mermaid` などのチルダフェンスでも図形描画をプレビュー・エクスポート対象にする。
- Lint 設定画面で、ワークスペース設定のオン/オフを切り替えても高度な設定画面へ自動遷移しないようにする。
- 一般設定は「無視 / 警告 / エラー」の見え方と診断重大度だけを扱う。
- ルールの有効/無効や詳細値は `.markdownlint.json` / `.markdownlint.jsonc` に保存し、必要に応じてファイル編集へ誘導する。
- グローバル設定（global settings）とワークスペース設定（workspace settings）を分け、ワークスペース設定が存在する場合はそれを優先する。
- 一般設定で「無視」を選んだルールは、ワークスペース側の KatanA 設定として保持し、`.markdownlint.json` の履歴を消さない。
- 一般設定で「無視」以外に戻した時は、保持していた詳細設定を `.markdownlint.json` に戻せるようにする。
- KML の設定読み込み契約を確認し、API が設定ファイルパスに対応していない場合でも、`.markdownlint.json` の内容を KML が要求する設定構造へ変換して渡す。
- 有効な Markdown ファイルの右クリックメニューに「ファイルをフォーマットする」を追加する。
- エクスプローラーの空き領域右クリックメニューに「ワークスペース内の Markdown を一括フォーマット」「ファイルの新規作成」「フォルダの新規作成」を追加する。
- エクスプローラーのフィルターの左に、ファイル追加とフォルダ追加のアイコンボタンを追加する。
- 追加アイコンは `katana-icon-management` に従い、各アイコンパック固有の SVG を取得して登録する。

## Capabilities

### Modified Capabilities

- `diagram-block-preview`: 図形描画フェンスとして ````` と `~~~` の両方を扱う。
- `workspace-shell`: エクスプローラーのファイル/フォルダ作成導線、空き領域コンテキストメニュー、Markdown フォーマット導線を追加する。

### New Capabilities

- `markdownlint-workspace-settings`: KatanA の Lint 一般設定、詳細設定、グローバル設定、ワークスペース設定、KML 設定入力の責務を整理する。

## Impact

- 主な影響範囲は `crates/katana-core/src/preview/section/fence.rs`、`crates/katana-core/src/markdown/fence/mod.rs`、`crates/katana-ui/src/linter_*_bridge.rs`、`crates/katana-platform/src/settings/types/linter.rs`、`crates/katana-ui/src/settings/tabs/linter/*`、`crates/katana-ui/src/views/panels/explorer/*`、`crates/katana-ui/src/views/panels/tree/*`、`crates/katana-ui/src/icon/types.rs`、`assets/icons/*`。
- KML のバージョン更新または API 変更が必要な可能性がある。
- `.markdownlint.json` を編集する機能は、外部エディタへ任せず、KatanA 側で読み込み・更新・保存の失敗を扱う必要がある。
- UI 変更を含むため、実装後はユーザーへ画面スナップショットを提示して確認を受ける。
