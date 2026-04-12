# Design: ショートカットUX全面再設計

## 1. ShortcutContext — コンテキスト境界の設計

### 1.1 Context Enum 定義

```rust
// crates/katana-ui/src/state/shortcut_context.rs (新規)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutContext {
    /// グローバル（どこでも発火）
    Global,
    /// エディタペインにフォーカスがある場合のみ
    Editor,
    /// プレビューペインがカレントの場合のみ
    Preview,
    /// エクスプローラー（サイドバー）がフォーカスを持つ場合
    Explorer,
    /// モーダル（コマンドパレット、検索、ファイル操作など）が開いている場合
    Modal,
    /// ショートカット録音中（このContext中は他のすべてのショートカットをブロック）
    Recording,
}
```

**コンテキスト優先順位（高い順）:**
`Recording > Modal > Editor > Preview > Explorer > Global`

### 1.2 フレームごとのコンテキスト判定

```rust
// ShortcutContextResolver
impl ShortcutContextResolver {
    pub fn resolve(state: &AppState, ctx: &egui::Context) -> ShortcutContext {
        // 1. 録音中は最高優先
        if is_recording_shortcut(ctx) { return ShortcutContext::Recording; }
        // 2. モーダル系（コマンドパレット, 検索モーダル, ファイル操作など）
        if state.layout.show_search_modal
            || state.command_palette.is_open()
            || state.layout.pending_close_confirm.is_some()
            || ... { return ShortcutContext::Modal; }
        // 3. エディタ（TextEditがフォーカス）
        if ctx.memory(|m| m.has_focus(editor_id)) { return ShortcutContext::Editor; }
        // 4. プレビュー（フルスクリーン/スライドショー含む）
        if state.layout.fullscreen || state.layout.slideshow { return ShortcutContext::Preview; }
        // 5. エクスプローラーフォーカス（将来拡張）
        // 6. Global
        ShortcutContext::Global
    }
}
```

### 1.3 CommandInventoryItem への context フィールド追加

```rust
pub struct CommandInventoryItem {
    pub id: &'static str,
    pub action: AppAction,
    pub group: CommandGroup,
    pub context: ShortcutContext,        // NEW
    pub label: fn() -> String,
    pub is_available: fn(&AppState) -> bool,
    pub default_shortcuts: &'static [&'static str],
}
```

**コンテキストマッピング（主要コマンド）:**

| コマンドID | Context |
|---|---|
| `app.settings` | `Global` |
| `file.*` | `Global` |
| `view.*` | `Global` |
| `edit.bold` / `italic` / etc | `Editor` |
| `help.*` | `Global` |

### 1.4 handle_shortcuts の再設計

```rust
// shell_ui_shortcuts.rs（全面再設計）
impl KatanaApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let active_context = ShortcutContextResolver::resolve(&self.state, ctx);

        // Recording中は一切ショートカットを発火しない（引数でrecording判定を除外）
        if active_context == ShortcutContext::Recording {
            return;
        }

        let os_bindings = self.state.config.settings.settings()
            .shortcuts.current_os_bindings();

        for cmd in CommandInventory::all() {
            // コンテキストチェック: Global は常に通過、その他は一致のみ
            if cmd.context != ShortcutContext::Global
                && cmd.context != active_context
            {
                continue;
            }
            if !(cmd.is_available)(&self.state) { continue; }

            // 以降はparse_shortcut + consume_shortcut（既存ロジック）
            ...
        }
    }
}
```

---

## 2. ショートカット設定UI再設計

### 2.1 行レイアウト変更

現状（3カラムGrid）:
```
[コマンド名] [shortcut文字列] [Editボタン]
```

新設計（2カラム + ペンアイコン）:
```
[コマンド名]  [OSアイコン付きキー表示] [🖊]
```
- 行全体がクリック可能（`ui.interact(row_rect, id, Sense::click())`）
- ペン（🖊）SVGアイコンは右端に最小サイズで配置
- クリックで「録音モーダル（ShortcutCaptureModal）」を開く

### 2.2 OSネイティブキーSVGアイコン

キー文字列の「primary」「shift」「alt」「mac_cmd」部分をOSアイコンに変換:

| トークン | macOS表示 | Windows/Linux表示 |
|---|---|---|
| `primary` / `command` | ⌘ (SVG) | Ctrl |
| `shift` | ⇧ (SVG) | Shift |
| `alt` | ⌥ (SVG) | Alt |
| `mac_cmd` （macOS固有） | ⌘ (SVG) | — |

実装: `shortcut_token_to_display(token: &str) -> ShortcutDisplayChip`

### 2.3 ShortcutCaptureModal — 録音専用モーダル

```
╔════════════════════════════════╗
║  「保存」のショートカットを入力してください  ║
║                                ║
║     [ Cmd + S ]               ║
║                                ║
║  Enter: 確定  Esc: キャンセル  ║
╚════════════════════════════════╝
```

- `egui::Window` または `Area` でオーバーレイ表示
- モーダル表示中: `ctx.set_key_filter()` で Esc/Enter 以外をOSレベルでブロック
- Esc → キャンセル（メモリクリア）
- Enter → `check_and_save_shortcut()` 呼び出し
- 入力されたキーをリアルタイムプレビュー表示

---

## 3. ShortcutAstLinter — 静的重複検知

### 3.1 アーキテクチャ

```
katana-linter/src/rules/domains/shortcut/
├── mod.rs          → ShortcutLinterOps::lint()
├── discovery.rs    → CommandInventoryParser（Rustソースを静的解析）
└── conflict.rs     → コンテキストを考慮した重複チェック
```

### 3.2 重複ルール

`{context, os, shortcut}` が同一の場合に違反:
- `Global` コンテキスト同士は完全重複禁止
- `Editor` と `Global` は重複許可（コンテキスト優先度で解決）
- `Editor` 同士は重複禁止（例: `primary+B[editor]` が2つあった場合エラー）

### 3.3 検知方法

`CommandInventory::all()` のソースコード（各 `*_commands.rs`）を `syn` でパースし、
`default_shortcuts` の文字列リテラルを抽出してルールを静的検査する。

将来的には `{action, context, os, shortcut}` の構造をソースに直接アノテーション
（例: `#[shortcut(context=Editor, os=all, key="primary+B")]`）する方向も検討。

---

## 4. デフォルトショートカット割り当て設計

コンテキスト設計が完成後、以下を付与する（B完成が前提）:

| コマンドID | 割り当て候補 | Context |
|---|---|---|
| `file.close_workspace` | `primary+Shift+W` | `Global` |
| `view.refresh_explorer` | `primary+Shift+R` | `Global` |
| `view.close_all` | `primary+Shift+K` | `Global` |
| `edit.strikethrough` | `primary+Shift+S` | `Editor` |
| `edit.heading1` | `primary+Shift+1` | `Editor` |
| `edit.heading2` | `primary+Shift+2` | `Editor` |
| `edit.heading3` | `primary+Shift+3` | `Editor` |
| `edit.bullet_list` | `primary+Shift+8` | `Editor` |
| `edit.numbered_list` | `primary+Shift+7` | `Editor` |
| `edit.blockquote` | `primary+Shift+.` | `Editor` |
| `edit.code_block` | `primary+alt+C` | `Editor` |
| `edit.horizontal_rule` | `primary+Shift+-` | `Editor` |
| `edit.insert_table` | `primary+alt+T` | `Editor` |
| `edit.ingest_image_file` | `primary+Shift+I` | `Editor` |
| `edit.ingest_clipboard_image` | `primary+alt+V` | `Editor` |
| `help.about` | （なし） | `Global` |
| `help.release_notes` | （なし） | `Global` |
| `help.user_guide` | （なし） | `Global` |

---

## 5. 移行戦略

### Phase 1: 既存コードの[editor]サフィックス
現在の `[editor]` サフィックスは `ShortcutContext::Editor` に対応する一時的な変換ルールとして維持し、
コンテキスト移行完了後に削除する。

### Phase 2: モーダルの key_pressed を統合
`views/modals/` 内の `key_pressed` 呼び出しは `ShortcutContext::Modal` の範囲内として
`handle_shortcuts()` に吸収する（段階的移行）。

### Phase 3: AST Linter の段階投入
Linter は Phase 2 完了後に有効化する（既存の競合が解消されてから）。
