## Context

KatanA には workspace file search があるが、これはファイル名ベースの発見に最適化されている。今回追加したいのは、Markdown 文書の本文中にある見出し、段落、コード断片を探すための content search である。
対象は current workspace 内の Markdown 文書に限定し、結果は検索語の一致だけでなく文脈付きスニペットと該当位置への遷移を必要とする。

この機能は UI、検索インデックス、文書ナビゲーションの 3 層にまたがるため、仕様だけでなく実装方針を先に固定しておく必要がある。

## Goals / Non-Goals

**Goals:**

- Markdown 文書本文を workspace 内で検索できるようにする
- 検索結果に文脈付きスニペットを出す
- 検索語を結果一覧、editor/code view、preview pane で強調する
- 検索語の履歴保存、履歴クリア、再利用をサポートする
- 検索語の次/前一致ジャンプをサポートする
- 検索結果選択で該当ファイルと該当箇所を開けるようにする
- 大きい workspace でも使えるように結果数を制限する
- 既存の file search と責務を分離する

**Non-Goals:**

- 全ファイル種別への一般化
- リモート検索や外部検索エンジンの導入
- 変更中の文書に対するリアルタイム全文インデックス同期の全面再設計
- search UI の大規模な再デザイン

## Decisions

### 1. 検索対象は Markdown に限定し、既存 file search とは別機能として扱う

本文検索とファイル名検索はユースケースが違うため、同一 UI の中で混ぜるより、capability として分けた方が説明しやすい。
`workspace-file-search` は「どのファイルを開くか」、本 change は「どこに書いてあるか」を担う。

- 採用理由:
  - ユーザーの期待が違う
  - インデックスの粒度が違う
  - 結果表示に必要な情報が違う
- 代替案:
  - 単一検索 UI に統合する: 実装は楽でも結果の意味が曖昧になるため不採用

### 2. 検索結果は「1結果 = 1一致位置」とする

結果が document 単位だと、どの hit に飛ぶかが曖昧になり、highlight と next / previous jump の責務も崩れる。
そのため 1 件の結果は「どのファイルか」ではなく「どの一致位置か」を表す単位にする。最小でも path、matched location、snippet を持つ。
matched location は既存の preview 側 position 扱いに合わせて、source line/column range と整合する表現を使う。

- 採用理由:
  - result selection の遷移先が一意になる
  - highlight と jump の基準を揃えられる
  - 他の AI が result contract を誤読しにくい
  - 既存の preview/editor 周辺コードに寄せて実装詳細を組み立てやすい
- 代替案:
  - 1結果 = 1 document: 複数 hit の扱いが曖昧になるため不採用
  - ファイル全文を結果に含める: 重く、UI が読みづらいので不採用

### 3. インデックスは workspace スコープで更新する

検索対象は workspace 内の Markdown 文書だけなので、インデックスも workspace スコープで持つ。
キャッシュの更新タイミングは既存の文書同期・ファイル監視の仕組みに合わせる。

- 採用理由:
  - スコープが明確
  - workspace 外のノイズを入れない
  - 既存の読み込み・保存フローと合わせやすい
- 代替案:
  - グローバル全文索引: 実装・無効化・再構築のコストが高いので不採用

### 4. 結果選択時は document open と matched location への移動を分ける

検索結果をクリックしたら、まず対象 Markdown を開く。その後、選択された result が指す matched location へカーソルまたはビューを移動する。
もしファイルが失われていたら open を止めて通知する。

- 採用理由:
  - open 失敗と位置移動失敗を分けて扱える
  - 既存の document open 経路を再利用しやすい
- 代替案:
  - 結果を直接エディタ内部に飛ばす: open 状態との整合が壊れやすいので不採用

### 5. 結果上限を設け、重い workspace での応答性を守る

上限は設定可能にしつつ、既定値を置く。結果は relevance 順か、少なくとも安定した順序で返す。
上限に達した場合は、UI 上で「さらに一致がある」ことを示す。

- 採用理由:
  - 大きい workspace での描画負荷を抑える
  - 結果の予測可能性を保つ
- 代替案:
  - 全件表示: UI と検索の両方で重くなるため不採用

### 6. 一致語の強調は snippet、editor/code view、preview pane の全てで行う

検索体験としては、結果一覧での視認性と、文書を開いた後の追跡性の両方が必要である。
そのため、検索 hit は検索結果の snippet だけでなく、開いた後の editor/code view と preview pane の両方で同じ一致語をハイライトする。

- 採用理由:
  - 一覧と本文閲覧の間で文脈を失いにくい
  - 見つけた単語を再探索せずに済む
  - 検索結果の意味が直感的になる
- 代替案:
  - 結果一覧のみ強調する: 開いた後に見失いやすいので不十分
  - preview のみ強調する: editor 側の追跡性が不足するので不十分
  - editor のみ強調する: preview 側の読書導線が不足するので不十分

### 7. 検索履歴は user-scoped の recent terms として保持する

検索ワードの再利用は workspace 固有ではなく、ユーザーの検索習慣に属する。
そのため履歴は user-scoped の recent terms として保持し、search UI の候補とする。履歴クリアはこの recent terms を空にする操作として提供する。

- 採用理由:
  - workspace を跨いでも再利用したい
  - UI での候補表示に適している
  - 実装がシンプルで、初版の scope に収まりやすい
- 代替案:
  - workspace-scoped 履歴: 別 workspace で再利用しにくい
  - 永続しない一時履歴: 再利用性が弱く、要件を満たしにくい

### 8. 次/前ジャンプは現在の検索語と active document を基準にする

検索語のジャンプ機能は、検索結果一覧とは別に、開いている Markdown 文書内の一致位置を巡回する操作として扱う。検索結果全体や workspace 全体は巡回対象にしない。
UI からは next / previous match action を呼び、現在の検索語に対する一致位置へカーソルまたはビューを移動する。

- 採用理由:
  - 「見つかった語を続けて読む」用途に合う
  - 検索結果一覧を毎回開かなくて済む
  - 既存の editor / preview ナビゲーションと繋げやすい
- 代替案:
  - 検索結果リスト上のみのジャンプ: 本文読書の導線として弱い
  - workspace 全体の hit 巡回: active document 基準が崩れ、挙動説明が難しくなるため不採用
  - 自動スクロールのみ: ユーザーの制御が弱いので不採用

## Risks / Trade-offs

- [Risk] インデックス更新が遅れると結果が古くなる -> Mitigation: 既存の保存・再読み込みイベントに合わせて更新し、必要なら手動再索引導線を用意する
- [Risk] snippet 抽出が雑だと検索価値が下がる -> Mitigation: result が指す matched location を基準に周辺文脈を切り出す
- [Risk] open 後の位置移動が失敗するとユーザーが迷う -> Mitigation: open 成功後に移動できなかった場合も通知を出す
- [Risk] 既存 file search と UI が競合する -> Mitigation: capability と command entry point を分け、役割を明確にする
- [Risk] ハイライトが強すぎると本文の可読性を落とす -> Mitigation: result list、editor、preview で同系色の控えめな強調に留める
- [Risk] 履歴が増えすぎると候補がうるさい -> Mitigation: recent terms に上限を設ける
- [Risk] next/previous ジャンプの対象が曖昧だと混乱する -> Mitigation: active document を基準にし、現在の検索語がない場合は動作を抑制する
