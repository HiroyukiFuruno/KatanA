## Context

Katana の UI では、現在「ワークスペース」と「エクスプローラー（ファイルツリー）」の役割が混在しています。

- サイドバーのトグルボタンは `FolderOpen` アイコンを使用し、ホバーテキストは「Workspace」となっている。
- 内部のコンテンツはファイルツリー（本来のエクスプローラー）と、最近開いたワークスペース（本来のワークスペース管理）の両方が含まれている。
これらを分離し、直感的な UI へと整理します。

## Goals / Non-Goals

**Goals:**

- サイドバーのアクティビティレールにおけるエクスプローラーのアイコンを「マルチファイル (Files)」アイコンに変更。
- ワークスペース履歴（管理）のアイコンを「フォルダ (FolderOpen)」アイコンに変更し、本来の「ワークスペース」としての役割を明確化。
- i18n 定義を `explorer_title` と `workspace_title` に分離。
- `crates/katana-ui` 内部の命名（State, Action, Directory）を「エクスプローラー」へとリファクタリング。

**Non-Goals:**

- ファイルツリーのドラッグ＆ドロップロジックや検索機能の内部実装の変更。
- `katana-core::workspace` 自体の機能変更（コンセプトの定義のみ）。

## Decisions

### 1. アイコンの刷新 (Icon Upgrade)

- **決定**: `assets/icons/files/explorer.svg` を新規作成し、エクスプローラーのトグルに使用する。現在の `Icon::FolderOpen` はワークスペース履歴（管理）のトグルへ移動する。
- **理由**: VSCode 等の主要な IDE に合わせることで、直感的な操作感を提供するため。また、Katana において「ディレクトリの保存＝フォルダ」というメタファーを確立するため。

### 2. State & Action のリネーム (State Renaming)

- **決定**: `LayoutState::show_workspace` を `show_explorer` に、`AppAction::ToggleWorkspace` を `ToggleExplorer` に変更する。
- **理由**: UI の名称変更に合わせて、実装コード側でも概念を統一するため。将来的に「本物のワークスペース管理画面」を独立させる際の混乱を防ぐ。

### 3. ディレクトリ構造の整理 (Path Refactoring)

- **決定**: `crates/katana-ui/src/views/app_frame/sidebar/workspace` を `explorer` にリネーム。
- **理由**: 実装ファイルの配置を UI の用語と同期させる。

## Risks / Trade-offs

- **[Risk]** → i18n のキー変更により、既存の翻訳が一時的に壊れる可能性がある。
- **[Mitigation]** → 日本語・英語だけでなく、全ての `.json` ローケルファイルを一括で更新する。
- **[Risk]** → 外部プラグインやテストが `show_workspace` に依存している場合、ビルドエラーになる。
- **[Mitigation]** → リファクタリング後に `cargo check` および統合テストを実行し、全箇所を修正する。
