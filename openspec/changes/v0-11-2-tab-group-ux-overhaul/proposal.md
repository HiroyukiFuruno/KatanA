# タブグループ UX 全面改善 (v0.11.1)

## Why

v0.11.0 で導入したタブグループ機能は、Chrome のタブグループと比較して UX が大幅に劣っている。具体的には：グループ作成時に名前入力が求められない、色選択 UI がない、グループヘッダーとタブの視覚的区別が不十分、グループの並び替えができない、グループがピン留めタブより左に寄らない、リネーム操作がコンテキストメニュー内の TextEdit で使いにくい。Chrome のタブグループに近い直感的な操作体験に引き上げる必要がある。

## What Changes

- **グループ作成フローの刷新**: 「新しいグループを作成」選択時にインライン名前入力を必須化し、色選択パレット（7色）を同時表示する
- **グループヘッダーの視覚的差別化**: グループヘッダーにアンダーラインバー（色付き）とドットインジケータを追加し、通常タブとの視覚的区別を明確にする。Chrome 風のコンパクトなチップデザインに変更
- **グループの並び順制御**: グループヘッダーのドラッグ＆ドロップによるグループ単位の移動を実装する。グループは所属メンバーごと移動する
- **グループのピン留め優先**: グループブロック全体をピン留めタブより左に配置する描画順序を導入する
- **リネーム UX の改善**: コンテキストメニュー内の TextEdit ではなく、グループヘッダーをダブルクリックでインラインリネームモードに入る方式に変更する
- **グループコンテキストメニューの拡充**: Ungroup（グループ解散）、Close Group（グループごと閉じる）、色変更パレットをコンテキストメニューに統合

## Capabilities

### New Capabilities

（なし — 既存機能の UX 改善のため新規 capability は不要）

### Modified Capabilities

- `tab-context-menu`: タブグループ操作（作成・追加・解除・色変更）のコンテキストメニュー仕様変更
- `document-organization`: グループの描画順序（ピン留めとの優先関係）、グループ単位移動、インラインリネームの仕様変更

## Impact

- `crates/katana-ui/src/views/top_bar/ui.rs` — グループヘッダー描画、インラインリネーム、ドラッグ＆ドロップ、描画順序の大幅改修
- `crates/katana-ui/src/app_state.rs` — 新規 Action（UngroupTabGroup, CloseTabGroup, ReorderTabGroup）追加
- `crates/katana-ui/src/app/action.rs` — 上記 Action のハンドラ実装
- `crates/katana-ui/src/state/document.rs` — TabGroup 構造体は変更なし（既存フィールドで対応可能）
- `crates/katana-ui/src/state/layout.rs` — インラインリネーム用の UI state 管理
- `crates/katana-ui/src/i18n/types.rs` + 各言語 YAML — 新規 i18n キー追加
- テストコード — 既存グループ関連テストの更新・追加
