# Markdown Preview Component

## Overview
`katana-markdown-preview` は、KatanAのUIから独立してMarkdownのパースとレンダリングを担当するクレートです。
eguiのネイティブ描画の限界（絵文字のフォールバック問題、Mermaid等リッチなダイアグラム対応の困難さ）を克服するため、内部的に WebView を用いて React 等でバンドルされた JS をロードして描画を行います。

## Requirements

### R1. Native Rust Rendering (No WebView)
- `wry` などの WebView ライブラリに頼らず、`egui` のネイティブ描画パイプラインを純粋なRust実装として維持する。
- macOS および Windows 双方において、システムネイティブのカラー絵文字が正確に表示されるよう、`egui` のテキスト描画パイプラインをハック（拡張）する。

### R2. Emoji Hack Integration
- `egui_commonmark` のパース処理に介入し、テキストから絵文字のUnicodeブロックを検知した際に、`egui::Image`（オープンソースのTwemoji等のSVG/PNGアセット）として動的に置換するロジックを組み込む。
- 必要に応じて、フォールバック用のOSネイティブフォントを自動取得する機能を備える。

### R3. API & Component Encapsulation
- `katana-ui` は `PreviewWidget` に Markdown テキストと必要な設定（`PreviewTheme` 等）を渡すのみとし、パースや描画（絵文字の置換を含む）のドメイン知識を `katana-markdown-preview` 内に完全にカプセル化する。

### R4. Workspace Cleanup
- `egui_commonmark` およびその関連依存の KatanA 独自パッチ（`vendor/` のフォーク）を独立した Git リポジトリへ退避させるか、このクレート内で隠蔽し、KatanA ワークスペースのルートからパッチ指定を完全に排除する。
