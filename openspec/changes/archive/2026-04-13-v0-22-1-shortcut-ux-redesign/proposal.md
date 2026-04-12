## Why

現在のショートカット設定UIには3つの根本的な課題がある。

**1. UX品質の低下（UI面）**
- 「Edit」ボタンが列を占有し視覚的にノイズが多い
- キー表示がプレーンテキストで「primary+S」など技術的な文字列が露出する
- 録音中（editing中）フォーカスの境界が曖昧で、他のUIへの誤操作を招く

**2. ショートカット実行の設計的欠陥（Context境界の欠如）**
- 現在 `handle_shortcuts()` はフレームごとに全コマンドを `is_available` チェック後に実行するが、
  「どこにフォーカスがあるか」という Context 概念が存在しない。
- エディタがカレントの場合に `primary+B`（Bold）と `view.explorer`（Toggle Explorer）が
  同一キーで競合するが、`[editor]` サフィックスによって回避している。
  この場当たり的なサフィックスは事前設計を欠いており、機能拡張と共にショートカット競合バグが多発する技術的負債である。
- コマンドパレット・モーダル・フルスクリーン・スライドショーなどが各自で `key_pressed` を呼んでおり、
  ショートカット処理が9箇所以上に分散している。

**3. 重複検知の欠如（Linter面）**
- ショートカットの重複は現在ランタイムの警告のみ（`shortcut_conflict` メモリ）で管理されている。
  本来は i18n linter のように、ショートカット定義自体を静的解析で重複を検知すべきである。

## What Changes

### A. ショートカット設定UI再設計（UX改善）
- 編集ボタンを廃止し、行全体またはペンSVGアイコンをクリックして「録音モーダル（専用入力画面）」を開く
- 録音中はアプリ全体の入力をescape/enter以外無効化（`request_focus` + `set_key_filter`）
- キー表示部分にOSごとのネイティブキーSVGアイコン（⌘/Ctrl/Win/Super）を追加
- Enterで確定、Escでキャンセル（OSごとの差異を吸収）

### B. コンテキスト境界を持つショートカット管理システム（大規模リファクタリング）
- `ShortcutContext` enum（`Global`, `Editor`, `Preview`, `Explorer`, `Modal`, `Recording`）の導入
- `CommandInventoryItem` に `context: ShortcutContext` フィールドを追加
- フレームごとにアクティブなコンテキストを判定する `ShortcutContextResolver` を実装
- `handle_shortcuts()` をコンテキスト認識型に書き換え、分散した `key_pressed` を統合
- `[editor]` サフィックスなどの場当たり的な仕組みを撤廃

### C. ショートカットAST Linter（静的重複検知）
- `katana-linter` に新ドメイン `shortcut` を追加
- `CommandInventory::all()` の `default_shortcuts` フィールドを静的解析し、コンテキストを考慮した重複をビルド時に検知
- `{action, context, os, shortcut}` の構造でルールを管理

### D. キー未割り当てコマンドへのデフォルト割り当て
- `file.close_workspace`, `view.refresh_explorer`, `view.close_all`,
  `edit.strikethrough`, `edit.heading1〜3`, `edit.bullet_list`,
  `edit.numbered_list`, `edit.blockquote`, `edit.code_block`,
  `edit.horizontal_rule`, `edit.insert_table`, `edit.ingest_image_file`,
  `edit.ingest_clipboard_image`, `help.*` のうち適切なものにショートカットを割り当て

## Capabilities

### New Capabilities

- `shortcut-context-routing`: フォーカスコンテキスト（Editor/Preview/Explorer/Modal等）を認識してショートカットを適切に発火する
- `shortcut-capture-modal`: 専用おモーダルでキー入力を補足し、OSネイティブアイコンで視覚化する
- `shortcut-ast-linter`: コンテキストを考慮したショートカット重複をビルド時に静的検知する

### Modified Capabilities

- `shortcut-settings-ui`: 編集ボタン廃止、行クリック/ペンアイコン対応、録音中の入力ブロック、OSアイコン表示
- `shortcut-execution`: コンテキスト境界を持つ統一ショートカットルーティングへの移行

## Impact

- 主な影響範囲:
  - `crates/katana-ui/src/state/command_inventory/` 全体（`context` フィールド追加）
  - `crates/katana-ui/src/shell_ui/shell_ui_shortcuts.rs`（全面再設計）
  - `crates/katana-ui/src/settings/tabs/shortcuts.rs` と `shortcuts_helpers.rs`（UI再設計）
  - `crates/katana-linter/src/rules/domains/`（`shortcut` ドメイン追加）
  - モーダル系ファイル群（`views/modals/`）の `key_pressed` 処理を統合
- B が完成しない限りDは実装不可（コンテキスト競合のリスク）
- B・Cは 既存の `[editor]` サフィックス互換を一時維持しながら段階的に移行する
