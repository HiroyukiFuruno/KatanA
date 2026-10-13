## 現状整理

現在のワークスペース UI は 2 箇所に責務が分かれている。

- `crates/katana-ui/src/views/app_frame.rs`
  - `show_workspace = false` のとき、`workspace_collapsed` として単一の `ChevronRight` トグル列だけを描画する
- `crates/katana-ui/src/views/panels/workspace/ui.rs`
  - ペイン表示時にタイトル文字列、履歴、更新、フィルター、検索、全展開、全閉を同じヘッダー周辺に描画する

現状解析で確定している事実は以下。

- 検索モーダル起動は `state.layout.show_search_modal = true` に集約されており、ショートカット (`Cmd+P`) とボタン導線は同じ state を使える
- 最近のワークスペース履歴は `settings.workspace.paths` を表示しており、`OpenWorkspace` / `RemoveWorkspace` action が既にある
- ワークスペース表示有無は `state.layout.show_workspace` の bool で完結しており、追加の layout state がなくてもレール化は可能
- 現状の collapsed 状態では、検索と履歴の導線が消える
- `crates/katana-platform/src/filesystem/scanner.rs` は sibling entry を `a.path().cmp(b.path())` で比較しており、`v0-11-x` が `v0-9-x` より前に来るような辞書順ソートになっている
- `crates/katana-ui/src/views/panels/workspace/ui.rs` は hierarchical tree しか描画しておらず、display mode を切り替える state も `...` メニューも存在しない
- 現在の tree row は basename 前提であり、flat 表示を file 単位にした場合は workspace-relative path を出さないと同名 file を区別できない

今回の change の本質は、新しい機能を増やすことではなく、既存 action と state を左レールへ再配置して UX を整理することにある。

## 目標 / 非目標

**目標:**

- ワークスペース表示切り替え・検索・履歴を、ペインの開閉と独立した左アクティビティレールへ移す
- ワークスペースペインのヘッダーからタイトル文字列を除去し、操作とツリー表示を優先する
- 更新/フィルターと全展開/全閉を 2 グループに分け、役割を判別しやすくする
- version-aware sort により `v0-9-x` と `v0-11-x` のような名前を自然順で表示する
- `...` メニュー経由で tree / flat 表示を切り替えられるようにする
- flat 表示では directory node を描かず、file 単位の一覧として扱えるようにする
- 既存 icon / action / state を最大限再利用し、表示モードは workspace ごとに永続化するが、新しい永続化フォーマットは増やさない
- 他の実装者が会話履歴なしで読んでも、変更対象 file と UI state が分かる状態にする

**非目標:**

- ワークスペースツリーのノード表示仕様や検索アルゴリズムの変更
- 新アイコン資産の導入
- 履歴保存形式や検索ショートカット仕様の変更
- フィルターの正規表現仕様や計算ロジックの変更
- workspace ごとの表示モード永続化の破壊的変更

## あるべき状態

この change 完了時点でのあるべき状態は次のとおり。

- 左端に常時表示の細いアクティビティレールがある
- `show_workspace = false` でも、ワークスペース再表示・検索・履歴の導線が残る
- ワークスペースヘッダーには `Workspace` / `ワークスペース` の文字列がない
- ヘッダー先頭側に更新/フィルター、末尾側に全展開/全閉がある
- ヘッダー末尾側には `...` メニューがあり、`表示 -> フラット表示` を toggle できる
- 検索はショートカットと左レールの両方から同じ `show_search_modal` を開く
- 履歴は `settings.workspace.paths` をそのまま使い、新しい persistence を持たない
- tree 表示では directory hierarchy を保ち、flat 表示では file 単位の一覧へ切り替わる
- flat 表示フラグの既定値は `false` であり、初期表示は tree になる。ユーザーが切り替えた表示モードは workspace ごとに保持される
- `v0-9-x` と `v0-11-x` のような数値付き名前は tree / flat の両方で自然順になる
- 実装途中でレイアウト制約が設計とずれた場合、先に OpenSpec artifact を更新してからコードを進める

## 設計判断

### 1. `workspace_collapsed` を専用アクティビティレールへ置き換える

`app_frame.rs` にある collapsed 用の単一トグル列は廃止し、常時表示の細い左レールへ置き換える。これにより、ペインを閉じても検索と履歴の導線が残る。

- 採用理由:
  - 現行の最大の UX 欠点は「閉じると頻出導線が消える」こと
  - `show_workspace` bool は既に存在し、レール常駐化に追加 state を要しない
- 代替案:
  - ペインヘッダー内の再配置だけで済ませる
  - これは collapsed 時の導線欠落を解消できないため不採用

### 2. レールは既存 action/state へ直接つなぐ

レール上の 3 ボタンは新しい action を増やさず、既存経路へ接続する。

- ワークスペース表示切り替え
  - `state.layout.show_workspace` を toggle
- 検索
  - `state.layout.show_search_modal = true`
- 履歴
  - `settings.workspace.paths` を使ってメニューを描画し、`OpenWorkspace` / `RemoveWorkspace` を dispatch

これにより、ロジック変更は最小化し、主な変更対象を UI 配置へ限定する。

### 3. ワークスペースヘッダーは「現ワークスペース固有操作」だけに絞る

ヘッダーからタイトル文字列、検索、履歴を外し、残す操作を 2 グループへ分離する。

- 先頭側グループ
  - 更新
  - フィルター
- 末尾側グループ
  - 全展開
  - 全閉

フィルター入力欄は現行どおりトグル直下に出し、`filter_enabled` / `filter_query` / `filter_cache` をそのまま使う。

### 4. 履歴ボタンは 0 件でも非活性で残す

履歴が 0 件でもボタンは配置したまま disabled にする。理由はレールのアイコン並びを固定し、muscle memory を崩さないため。

### 5. 実装設計: 変更対象と責務

最低限の責務分割を以下で固定する。

- `crates/katana-ui/src/views/app_frame.rs`
  - 左アクティビティレールの枠を描画
  - `show_workspace` の true/false に応じて、ペイン本体の有無だけを切り替える
- `crates/katana-ui/src/views/panels/workspace/ui.rs`
  - タイトル文字列を除去
  - ヘッダーのボタン群を 2 グループへ再配置
  - `...` メニューと `表示 -> フラット表示` toggle を追加
  - 検索/履歴 UI を削除
- `crates/katana-ui/src/views/panels/workspace/logic.rs`
  - tree / flat の表示 projection を切り替える
  - flat 表示用 file list の workspace-relative path 整形と sort を担当する
- `crates/katana-ui/src/state/layout.rs`
  - `show_workspace` / `show_search_modal` を既存のまま使う
- `crates/katana-ui/src/state/workspace.rs`
  - workspace ごとの flat 表示フラグを保持する
- `crates/katana-platform/src/filesystem/scanner.rs`
  - sibling entry に対する version-aware sort comparator を持つ
- `crates/katana-ui/locales/*.json`
  - レールの hover text が既存文言で不足する場合のみ追加

実装順序は次を推奨する。

1. `app_frame.rs` の collapsed 列をレールへ置き換える
2. `scanner.rs` と workspace projection 側に共有の version-aware sort を導入する
3. `workspace/ui.rs` ヘッダーから検索/履歴を外し、2 グループへ再配置し、`...` メニューを追加する
4. flat 表示用 file list projection と workspace-relative path 表示を追加する
5. レールの検索/履歴ボタンを既存 state/action に接続する
6. no-workspace / workspace-open / workspace-collapsed / flat display の各状態を回帰確認する

### 6. 一覧順序は version-aware sort を共有 comparator で統一する

tree 表示と flat 表示で sort 結果がズレると、同じ workspace を別表示で見たときに順序の予測可能性が崩れる。
そのため comparator は 1 箇所に寄せ、少なくとも以下を満たす。

- `v0-9-x` は `v0-11-x` より前に来る
- 数値 token は文字列比較ではなく数値比較する
- tree 表示では directory を file より先に出す
- flat 表示では file のみを対象に、workspace-relative path で安定 sort する

- 採用理由:
  - 並び順の期待を UI 全体で揃えられる
  - scanner と flat projection で sort 実装が分岐するのを防げる
- 代替案:
  - tree は辞書順、flat は自然順: 表示モードごとに順序が変わるため不採用
  - flat 表示だけ後段で sort する: tree 側の `v0-11-x` 問題が残るため不採用

### 7. flat 表示は file-only / workspace-relative path 表示にする

flat 表示は「directory を畳む tree」ではなく、「directory node を持たない file 一覧」として扱う。
そのため row は basename ではなく workspace-relative path を表示し、同名 file でも区別できるようにする。directory node が存在しないため、expand all / collapse all は flat 表示中に disabled とする。

- 採用理由:
  - 「ディレクトリの概念がない file 単位の flat 表示」という要件に一致する
  - 同名 file の識別が可能になる
  - tree 固有の expand/collapse 操作が flat 表示で誤解を生まない
- 代替案:
  - basename だけ表示する: 同名 file を区別できないため不採用
  - directory を 1 階層だけ残す: flat 表示要件を満たさないため不採用

### 8. 表示モードは `WorkspaceState` に置く workspace-local な `flat display` bool とする

表示モードは workspace pane 専用の UI state であり、workspace をまたいで共有する必要はない。そのため `WorkspaceState` に workspace-local な `flat display` bool として持たせ、既定値は `false`、`false = tree`、`true = flat` とする。workspace を再度開いた時は、最後に選ばれた値を復元する。

- 採用理由:
  - `default: false` をそのまま実装に落とし込める
  - 既定値 tree をシンプルに保証できる
  - workspace ごとの見やすさを維持できる
  - `show_workspace` と独立して pane 表示のまま切り替えられる
- 代替案:
  - `LayoutState` に置く: workspace 固有 state と UI 全体 state が混ざるため不採用
  - session-local のみで持つ: workspace ごとの選択を復元できないため不採用

### 9. `...` メニューに `表示 -> フラット表示` の checkable item を置く

flat 表示への入口はワークスペースヘッダーの `...` メニューとし、その中の `表示` サブメニューに `フラット表示` を置く。
flat が有効な時は `✔ フラット表示` のように active 状態が即座に分かる表現にし、再度選択すると tree へ戻る toggle とする。

- 採用理由:
  - user request の導線に一致する
  - ヘッダーを常用ボタンで過密にせず、表示関連の操作を集約できる
  - 将来の表示系 option を同じ menu に追加しやすい
- 代替案:
  - 独立トグルボタンをヘッダーに置く: 常用ボタンが増えすぎるため不採用
  - settings 画面へ移す: 即時切り替え用途として遠すぎるため不採用

### 10. flat 表示でも filter は同じ正規表現契約を維持する

filter は workspace tree 専用機能として別解釈せず、「現在の workspace 一覧に対する絞り込み」として扱う。
つまり filter set を先に求め、その結果を tree / flat のどちらで投影するかだけを表示 mode で切り替える。

- 採用理由:
  - filter の意味が表示 mode で変わらない
  - 既存 `filter_enabled` / `filter_query` / `filter_cache` を再利用しやすい
- 代替案:
  - flat 表示だけ別 filter 実装にする: UX と実装が分岐するため不採用

### 11. 前提が崩れた場合は artifact を先に更新する

以下の条件では、実装者は先に artifact を更新してから次へ進む。

- `app_frame.rs` の panel 構造上、レールを別 panel に分離しないと安定しないと分かった場合
- 履歴 0 件で disabled 表示よりも非表示の方が妥当だと判断できる具体的理由が出た場合
- ヘッダー 2 グループ化でフィルター入力欄の現位置維持が困難と分かった場合
- flat 表示で workspace-relative path 以外のラベル形式が必要だと判明した場合
- expand/collapse の disabled 表示が UI 上不自然で、非表示化へ変更すべき具体的理由が出た場合

是正フロー:

1. 制約や試作結果を `design.md` に追記する
2. 影響する requirement を `specs/*/spec.md` で修正する
3. 実装順序や検証項目が変わるなら `tasks.md` を更新する
4. その後にコード実装へ戻る

### 12. アクティビティレールのアイコン並び順と Drag & Drop 永続化

左アクティビティレールのアクションアイコン（ワークスペース切替、検索、履歴）は、ユーザーの好みで Drag & Drop による並べ替えを可能とする。
並び順は `LayoutSettings` に `activity_rail_order` (`Vec<ActivityRailItem>`) として永続化する。

- `ActivityRailItem` は `ExplorerToggle`, `Search`, `History` の enum とする。
- 既定の並び順は、フィードバックに基づき `[History, ExplorerToggle, Search]` とする（履歴からの切り替えを最上部へ）。
- `app_frame.rs` のレール描画ロジック内で、この配列順に従ってボタンを描画すると同時に `egui::Sense::click_and_drag()` 等を用いた並べ替え UI を提供する。

## リスク / トレードオフ

- **[Risk] 左レール追加で横幅が減る**
  - Mitigation: 幅は collapsed 列と同程度の固定最小幅に抑え、タイトル文字列削除で相殺する
- **[Risk] `ChevronLeft/Right` は意味が直感的でない**
  - Mitigation: active fill と tooltip で現在状態を補う
- **[Risk] レールとヘッダーに責務が割れて導線が曖昧になる**
  - Mitigation: レールは主要導線、ヘッダーは現ワークスペース固有操作、と責務を固定する
- **[Risk] natural sort comparator の追加で scanner 実装が複雑になる**
  - Mitigation: comparator を utility として切り出し、tree / flat の双方で共有する
- **[Risk] flat 表示で path ラベルが長くなり、可読性が落ちる**
  - Mitigation: workspace-relative path を前提にしつつ truncate で収め、tooltip で全体を補う
- **[Risk] workspace ごとの表示モードが settings 側の保存形式変更を伴う**
  - Mitigation: 既存の workspace 設定に追記するだけに留め、破壊的変更は避ける

## 移行方針

- workspace ごとの表示モードは永続化する
- 既存ショートカット、履歴更新順、ワークスペース action は維持する
- UI 導線に加えて、一覧 sort comparator と view projection を追加する
