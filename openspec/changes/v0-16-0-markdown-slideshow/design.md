# 設計: Markdown スライドショー表示

## 現状整理

現在の Markdown 関連 UI は、editor と preview の通常表示を中心に構成されている。

- `crates/katana-ui/src/views/panels/preview.rs`
  - Markdown preview の制御群とレンダリング表示を持つ
- `crates/katana-ui/src/preview_pane/fullscreen.rs`
  - 画像向けの全画面 viewer が既に存在する
- `crates/katana-core/src/markdown/*`
  - Markdown レンダリング、diagram、export、theme preset が分かれている

現状解析で確定している事実は以下。

- 現在の theme 色は `katana_platform::theme::ThemeColors` と Markdown 側の color preset から解決されている
- preview には既に fullscreen 系の UI state と終了導線の考え方がある
- Markdown の diagram block は core 側で個別レンダラを通して扱われている
- 印刷/export 系のページ分割ロジックがあるなら、その契約を再利用するのが最も整合的である

今回の change の本質は、Markdown preview を新しい表示モードとして拡張し、発表用途に必要な閲覧フローを追加することにある。

## 目標 / 非目標

**目標:**

- アクティブな Markdown 文書を全画面スライドショーとして表示できる
- Markdown 系の制御群からスライドショーを起動できる
- 左右のページングで前後のページへ移動できる
- `Esc` と右上 `[x]` で確実に終了できる
- 現在の theme をそのまま反映する
- diagram を含むページ分割は自動レイアウトに委ねる
- 他の実装者が会話履歴なしで読んでも、どの state と renderer を触るか分かる状態にする

**非目標:**

- Markdown 編集体験そのものの変更
- 印刷機能の UI 追加
- 新しいテーマ設定の追加
- スライドテンプレートや手動レイアウト編集
- local LLM による要約や自動発表生成

## あるべき状態

この change 完了時点でのあるべき状態は次のとおり。

- Markdown preview の制御群からスライドショーを開ける
- 開いたスライドショーは全画面表示になる
- 左右キーまたは同等のページング導線で前後へ移動できる
- 右上 `[x]` と `Esc` で同じ終了動作になる
- スライドショーの色味は current theme と一致する
- diagram を含むページも自動でページ分割され、表示が崩れない

## 設計判断

### 1. 起動導線は Markdown 系の制御群に置く

スライドショーは preview に対する閲覧モードの一種なので、Markdown 系の制御群にボタンを追加する。別メニューや settings へ逃がすと、通常の Markdown 確認導線から遠くなる。

- 採用理由:
  - preview と同じ文脈で起動できる
  - ユーザーが「今見ている Markdown をそのまま発表モードにする」操作だと理解しやすい
- 代替案:
  - 画面全体の共通メニューに置く: Markdown 専用操作として弱い

### 2. 表示は全画面モーダルまたは専用 viewer で実現する

通常 preview を拡張するより、全画面専用 viewer として分離した方が、終了導線とページングを安定して実装できる。

- 採用理由:
  - `Esc` と `[x]` の終了導線を単純化できる
  - 通常 preview のレイアウトを壊さない
  - 発表用途に必要な没入感を作りやすい
- 代替案:
  - preview pane をそのまま拡大する: 操作系と表示系が混ざりやすい

### 3. ページングは Markdown のページ列を基準にする

スライドショーの前後移動は、ページ単位のインデックスで扱う。ページ列の生成は、印刷と同じページ分割契約を再利用する。

- 採用理由:
  - 印刷結果と同じ切れ方を保てる
  - 章や diagram のまとまりを壊しにくい
- 代替案:
  - heading 単位のみで区切る: diagram や長文で破綻しやすい

### 4. diagram の切れ目は自動ページ分割に任せる

diagram block を手動で固定位置に詰めるのではなく、印刷時と同じ自動ページ分割に委ねる。これにより、見た目の整合性を保ちつつ、個別の diagram ルールを増やさずに済む。

- 採用理由:
  - 既存の印刷ロジックとの整合が高い
  - diagram を含む文書で手動調整コストを増やさない
- 代替案:
  - diagram block 専用のページ制御を作る: 実装と保守が重くなる

### 5. theme は preview と同じ解決経路を使う

スライドショーは別テーマを持たず、現在の theme state をそのまま解決する。色の決定は preview と同じ経路を共有する。

- 採用理由:
  - preview と見た目を揃えられる
  - 設定項目を増やさずに済む
- 代替案:
  - スライドショー専用 theme を作る: 変更面積が広がる

### 6. 終了導線は `Esc` と `[x]` のみに限定する

終了の仕方を 2 つに絞ることで、誤操作を減らし、閉じ方を明確にする。右上 `[x]` はモーダルの閉じる操作として視覚的に分かりやすく、`Esc` はキーボード操作の基準になる。

- 採用理由:
  - ユーザーが迷わない
  - 他のビューアとの挙動を揃えやすい
- 代替案:
  - ツールバーに複数の終了ボタンを置く: 役割が曖昧になる

### 7. 実装対象と責務

最低限の責務分割を以下で固定する。

- `crates/katana-ui/src/views/panels/preview.rs`
  - Markdown 制御群にスライドショー起動ボタンを追加する
- `crates/katana-ui/src/preview_pane/fullscreen.rs`
  - Markdown スライドショーの全画面 viewer を描画する
- `crates/katana-ui/src/preview_pane/ui.rs`
  - 開始・終了・ページング state を管理する
- `crates/katana-core/src/markdown/*`
  - ページ列生成と diagram を含む自動ページ分割の契約を担う
- `crates/katana-ui/src/state/*`
  - スライドショーの表示 state と active document 参照を管理する

### 8. 前提が崩れた場合は artifact を先に更新する

以下の条件では、実装者は先に artifact を更新してから次へ進む。

- 印刷用ページ分割ロジックをそのまま再利用できないと分かった場合
- fullscreen モーダルの構造上、preview pane から分離しないと安定しないと分かった場合
- theme 継承に追加の state が必要だと分かった場合
- diagram の自動ページ分割が preview と一致しないと判明した場合

是正フロー:

1. 制約や試作結果を `design.md` に追記する
2. 影響する requirement を `specs/*/spec.md` で修正する
3. 実装順序や検証項目が変わるなら `tasks.md` を更新する
4. その後にコード実装へ戻る

## リスク / トレードオフ

- **[Risk] 全画面表示で通常操作が見えなくなる**
  - Mitigation: `Esc` と `[x]` を常時見える終了導線として固定する
- **[Risk] ページ分割が印刷とずれる**
  - Mitigation: 印刷側と同じページ生成契約を使う
- **[Risk] theme 継承の責務が UI と core にまたがる**
  - Mitigation: 色の決定を既存 preview の theme 解決経路へ寄せる
- **[Risk] diagram を含む文書でページが不自然に切れる**
  - Mitigation: diagram 専用の手動補正ではなく、自動ページ分割を優先する
