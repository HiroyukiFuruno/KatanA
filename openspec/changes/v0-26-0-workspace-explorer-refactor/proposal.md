## Why

現在の Katana UI では、ファイルを表示する機能（エクスプローラー）が「ワークスペース」と命名されており、本来の「ディレクトリの集合・保存を司るワークスペース」という概念と混同されています。この不一致を解消し、i18n、ファイル名、実装、アイコンを含めた一貫性のあるリファクタリングを行うことで、UX の向上とコードの関心の分離（Separation of Concerns）を実現します。

## What Changes

- **用語の統一**: UI 上の「ワークスペース」を「エクスプローラー」に改称（i18n を含む）。
- **アイコンの刷新**: サイドバーのエクスプローラーボタンに「マルチファイル」アイコンを導入し、フォルダアイコンを本来の「ワークスペース管理」の象徴として再配置します。
- **実装レベルのリファクタリング**: `LayoutState`や `AppAction`における命名、およびサイドバー関連のディレクトリ構造を `explorer` へと整理します。

## Capabilities

### New Capabilities
- `explorer-sidebar`: ファイルツリー閲覧と操作に特化したサイドバー機能の再定義。
- `workspace-management`: 複数ディレクトリの管理、履歴管理としての「ワークスペース」機能。

### Modified Capabilities
- `layout-state`: `show_workspace` から `show_explorer` への表示状態管理の変更。

## Impact

- `crates/katana-ui`: UI コンポーネント、i18n、状態管理、アイコン。
- `crates/katana-platform`: 設定サービスにおける最近のワークスペース履歴の扱い。
