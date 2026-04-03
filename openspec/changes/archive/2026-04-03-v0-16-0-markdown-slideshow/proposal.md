# OpenSpec Change Proposal: Markdown スライドショー表示 (v0.17.0)

## 背景

KatanA の Markdown preview は、通常表示では編集内容の確認に十分だが、説明資料やデモ用途では「全画面で順に見せる」導線がない。アクティブな Markdown をそのままスライドショーとして表示できれば、編集・確認・発表の切り替えが滑らかになる。

## 変更内容

- アクティブな Markdown 文書を全画面スライドショーで表示する機能を追加する。
- 起動導線は Markdown 系の制御群に追加し、preview から直接入れるようにする。
- スライドショー中は左右のページングで前後に移動できるようにする。
- 終了導線は `Esc` と画面右上の `[x]` に限定し、閉じ方を明確にする。
- スライドショーは現在の theme を引き継ぎ、preview と色の整合を保つ。
- ダイアグラムを含むページ分割は、印刷時と同様に自動レイアウトへ委ねる。

## ケイパビリティ

### 追加されるケイパビリティ

- `markdown-slideshow`: アクティブ Markdown を全画面スライドショーとして表示し、ページングと終了導線を提供する。

### 変更されるケイパビリティ

- `markdown-preview`: preview の Markdown 制御群にスライドショー起動ボタンを追加する。
- `markdown-rendering`: 印刷と同じページ分割ルールを使って、スライドショー用のページ列を生成する。
- `theme-management`: スライドショーが現在の theme をそのまま継承するようにする。

## 影響範囲

- 主な影響範囲は `crates/katana-ui/src/views/panels/preview.rs`、`crates/katana-ui/src/preview_pane/*`、`crates/katana-core/src/markdown/*`、`crates/katana-ui/src/app/action.rs`、`crates/katana-ui/src/state/*`。
- 既存の画像フルスクリーンや preview の theme 解決ロジックは再利用し、Markdown 専用のスライドショー表示へ拡張する。
- 新しい編集モデルは導入せず、現在アクティブな Markdown 文書をそのまま表示対象にする。
