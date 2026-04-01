# Design: タブグループ UX 全面改善

## Context

v0.11.0 の tab group 実装は最小機能として動作するが、Chrome のタブグループ UX と比較して以下の問題がある：

1. **作成フロー**: デフォルト名「New Group」＋ハードコード灰色で即座に作成される。ユーザーに名前の考慮を促さない
2. **視覚的差別化**: グループヘッダーが `egui::Button` で描画され、通常タブと見分けがつかない
3. **移動不可**: グループヘッダーに drag 機能がなく、グループ全体の並び替えができない
4. **描画順序**: グループブロックは `open_documents` の先頭メンバー位置に anchored されるのみで、ピン留めタブとの関係性が未定義
5. **Rename UX**: コンテキストメニュー内の `text_edit_singleline` は、メニューが閉じると変更が反映されない問題を含み、操作性が悪い
6. **色変更**: コンテキストメニューに色変更 UI が存在しない

現在のアーキテクチャ：

- `TabGroup { id, name, color_hex, collapsed, members: Vec<String> }` — path 文字列ベースのメンバーシップ
- `DocumentState.tab_groups: Vec<TabGroup>` — グループのリスト
- `top_bar/ui.rs` の `TabBar::show()` — 描画ロジック（`group_first_indices` HashMap で先頭メンバーの前にヘッダーを挿入）
- `AppAction` enum — CreateTabGroup, AddTabToGroup, RemoveTabFromGroup, RenameTabGroup, RecolorTabGroup, ToggleCollapseTabGroup

## Goals / Non-Goals

**Goals:**

- Chrome のタブグループに匹敵する直感的な UX を提供する
- グループ作成時に名前入力を必須化し、色パレットを提示する
- グループヘッダーを視覚的にタブと明確に区別する（下線バー + コンパクトチップ）
- グループ単位のドラッグ＆ドロップ移動を実装する
- グループブロックをピン留めタブより左に描画する
- ダブルクリックによるインラインリネームを実装する
- コンテキストメニューに Ungroup / Close Group / 色変更パレットを統合する

**Non-Goals:**

- タブのグループ間ドラッグ＆ドロップ（個別タブの移動は既存機能で対応）
- グループのネスト（Chrome も非対応）
- グループ単位の保存・復元の仕様変更（既にセッション永続化で対応済み）

## Decisions

### D1: グループ作成フロー — インラインモーダルではなくコンテキストメニュー内パネル

**決定**: コンテキストメニューで「新しいグループを作成」を選択すると、名前入力欄と色パレットを含むサブメニューパネルを表示する。名前が空の場合は作成ボタンを無効化する。

**理由**: egui のモーダル制約（`egui::Window` は z-order 管理が限定的）を考慮し、コンテキストメニューのサブメニュー拡張で実装する方が安定性が高い。Chrome の「名前入力→Enter→即時作成」に近い操作感を実現する。

**代替案**: `layout.rs` に専用モーダル state を追加する方式。しかし egui のフレーム描画サイクルとの同期が複雑になるため却下。

### D2: グループヘッダーの視覚デザイン — コンパクトチップ + 下線バー

**決定**: グループヘッダーを以下のデザインで描画する：

- 縮小されたフォントサイズ（通常タブの 85%）でグループ名を表示
- グループカラーの円形ドット（●）を名前の左に配置
- メンバータブの下部にグループカラーの 2px アンダーラインを描画
- 折りたたみ時はドット + 折りたたみアイコン（▸）のみ表示

**理由**: Chrome のタブグループヘッダーは通常タブより小さく、色ドット + 名前というミニマルなデザイン。egui での再現に最適な構成。

### D3: 描画順序 — projection ベースの並び替え

**決定**: `TabBar::show()` の描画ループを以下の3フェーズに分割する：

1. **Phase 1**: グループブロック（グループヘッダー + 所属メンバー）— `tab_groups` の Vec 順で左から
2. **Phase 2**: ピン留めタブ（グループ未所属）
3. **Phase 3**: 通常タブ（グループ未所属 + ピン留めなし）— `open_documents` 順

各フェーズ内の order は既存の `open_documents` index に従う。

**理由**: Chrome ではグループが最も左に固定され、次にピン留め、最後に通常タブという順序。この3フェーズ描画で既存の tab drag-drop ロジックとの干渉を最小化できる。

### D4: グループ移動 — グループヘッダーの drag でグループ順序を変更

**決定**: グループヘッダーに `Sense::click_and_drag()` を設定し、drag 操作でグループ間の順序を変更する。`DocumentState.tab_groups` の Vec 内位置がそのまま描画順序となる。新規 `ReorderTabGroup { from: usize, to: usize }` Action を追加する。

**理由**: 既存の tab drag-drop と同じパターンで、`tab_groups` Vec の並び替えで完結する。

### D5: インラインリネーム — グループヘッダーのダブルクリック

**決定**: グループヘッダーをダブルクリックすると、ヘッダーが `TextEdit` に置き換わる。Enter キーまたはフォーカス離脱で確定する。`layout.rs` の `rename_tab_group_modal: Option<(usize, String)>` を `inline_rename_group: Option<(String, String)>` (group_id, current_text) に変更する。

**理由**: コンテキストメニュー内の TextEdit は操作性が悪い（ユーザーフィードバック #6）。ダブルクリック → インライン編集は Chrome と同じパターン。

### D6: 色パレット — 7色固定

**決定**: 以下の 7 色を固定パレットとして提供する（現行のテーマ対応カラー）：

- `#4A90D9` (Blue), `#D94A4A` (Red), `#4AD97A` (Green), `#D9A04A` (Orange), `#9B59B6` (Purple), `#F1C40F` (Yellow), `#1ABC9C` (Teal)

コンテキストメニュー内に横並びの色付き円ボタンとして表示する。

**理由**: Chrome も 8 色固定パレット。自由色選択は egui の制約上複雑になるため、固定パレットで十分。

## Risks / Trade-offs

- **[描画順序変更の複雑さ]** → 3フェーズ描画に切り替えることで、既存の drag-drop ロジック（`compute_drop_points`, `resolve_drag_drop`）の index 計算がずれるリスク。→ projection mapping 関数を導入して logical index ↔ display index を変換する
- **[コンテキストメニュー内パネルの UX 制約]** → egui のサブメニューは自動クローズが発生しやすい。→ `ui.memory` で一時的にメニュー固定フラグを管理する
- **[インラインリネームとクリック/ダブルクリックの競合]** → シングルクリック（collapse toggle）とダブルクリック（rename）の区別が必要。→ egui の `double_clicked()` を先に判定し、true ならシングルクリック処理をスキップ
