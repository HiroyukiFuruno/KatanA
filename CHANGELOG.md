# Changelog

All notable changes to KatanA Desktop will be documented in this file.

## [0.0.1] - 2026-03-16

### ♻️ Refactoring

- Drawio_renderer のclippy警告を修正
- テストを src/ から tests/ ディレクトリに移行し、Clippy を厳格化
- Katana-ui を lib/binary 構造にリファクタリングし、ロジックを抽出
- マジックナンバーを用途明確な名前付き定数に抽出
- 言語定義をlocales/languages.jsonに外部化
- Span_locationの重複をフリー関数に統合(自己レビュー修正)
- Egui描画ロジックとイベントルーティングの分離
- ソースコードとテストの日本語コメント・文字列を英語化
- UIレイアウト改善とリンターモジュール追加

### 🐛 Bug Fixes

- Clippy 警告・フォーマット・30行制限の修正
- スクリーンショットで確認した問題を修正
- PLANTUML_JAR を排他的オーバーライドにしてテストを安定化
- 3問題を修正 — レイジーロード・Mermaidフォント・デスクトップ強制移動
- スナップショットテストのフレーキー問題を修正
- Eguiのレイアウト制約を回避するためリスト内のコードブロックを前処理でデインデントする
- MacOS sed互換性のためInfo.plist更新をPerlに変更

### 📚 Documentation

- Mark test compilation, tab UI, and plantuml macos bug as done
- Coding-rulesにi18n規約（セクション11）を追加
- README・ドキュメントテンプレート追加、.obsidianをgitignore対象に変更
- プロジェクト基盤ファイル追加 — LICENSE(MIT)、README、開発環境セットアップスクリプト
- ADR(Architecture Decision Records)と統合テストシナリオを追加
- 技術的負債メモ(TECHNICAL_DEBT.md)を追加
- Organize-documentsのopenspecを追加
- 共通ドキュメント周りの英語化と日本語版(*.ja.md)の並行整備およびopenspecアーカイブ
- プロジェクト名をKatanAに統一（README、Cargo.toml、設定コメント）
- ドキュメントを一般配布向けに再構築 (#21)
- 「What is KatanA」セクションを追加（英語/日本語）
- KatanAの末尾「A」= Agentの由来を追記

### 🔧 Miscellaneous

- Bootstrap katana repository
- Remove opsx prompt files
- Align gitignore with official templates
- Task 6.2 完了マーク — bootstrap-katana-macos-mvp 全タスク完了
- Openspecディレクトリをgit管理から除外
- Gitignore更新（openspec, obsidian設定, katana-core .gitignore統合）
- 不要なドキュメントテンプレートとREADMEを削除
- CI カバレッジジョブ追加と品質ゲートの明文化
- Desktop-viewer-polishに向けたCI要件の厳格化と不要アセット削除
- Lefthookの検証コマンドをMakefileに統合・自動修正化
- 依存関係の更新 (dirs-sys 0.5.0, rfd 0.17.2, egui_commonmark features追加)
- GitHub Sponsors用のFUNDING.ymlを追加
- Cliff.tomlからCI botコミットを除外する設定を追加

### 🚀 Features

- Bootstrap Katana macOS MVP — Rust プロジェクト基盤と全コアモジュールの実装
- Task 3.2 — ネイティブ Markdown プレビューペイン実装
- I18n support, language setting, appAction expansion, bin rename
- ダイアグラムレンダリング改善（drawio矢印対応、mermaid PNG出力、CommandNotFound区別）
- ファイルシステムサービス拡張（ワークスペースツリー・ファイル監視改善）
- タブ別プレビュー管理、スクロール同期、macOSネイティブメニュー、ワークスペースパネル制御
- 検証強化 — lefthook導入、テスト追加、Clippy厳格化、品質ゲート定義
- AST Linter(katana-linter)を導入 — i18nハードコード文字列・マジックナンバー検知
- Apply Katana app icon and version for native About panel (#15)
- 設定の永続化基盤を実装（JsonFileRepository + SettingsService）
- ワークスペース・言語変更時に設定を自動保存
- 起動時に保存済み設定（ワークスペース・言語）を復元
- プレビュー機能改善 (画像パス解決、セクション分割の先頭フェンス対応、ダイアグラムレンダラー改善)
- About画面の改善とアプリ表示名KatanAへの統一
- MacOSアプリバンドル(.app)パッケージングの追加 (#18)
- MacOS DMGインストーラー生成の追加 (#19)
- リリース自動化（git-cliff + make release） (#20)
- リリースCDワークフロー(.github/workflows/release.yml)を新設 (#22)
- GitHub SponsorsのURL設定とREADME日本語版の追加

### 🧪 Testing

- Task 6.2 — プレビュー同期テスト追加
- Add app state unit tests and fix java headless mode for plantuml
- プレビュー同期のユニットテストを追加（タスク3.2完了）
- カバレッジ厳格化 — ignore-filename-regex 撤去・#[coverage(off)] 全廃・Regions 100% 強制
- LLVMカバレッジ算出の差異対応とテスト100%ゲートの厳密化
- 永続化ラウンドトリップの統合テストを追加


