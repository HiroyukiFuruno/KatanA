## Context

現在の KatanA には workspace file search modal があり、`SearchState` も file-path search 前提の shape になっている。一方で `AppAction` 側には設定切替、workspace 操作、view mode 変更など多くの command が既にあり、これらは top bar、menu、shortcut に散っている。
さらに `v0.12.0` で Markdown content search を入れる前提だと、navigation の入口は今後さらに分散する。

そのため `v0.15.0` では、既存 command と search provider を束ねる keyboard-first な command palette を追加する。

## Goals / Non-Goals

**Goals:**

- 1 つの palette から command execution、file open、Markdown content navigation を行えるようにする
- keyboard-first で高速に操作できるようにする
- existing `AppAction` 群を再利用し、command dispatch の source of truth を増やさない
- palette を主要な高速導線としつつ、既存 file-search modal を即時に壊さない

**Non-Goals:**

- local LLM を使った自然言語コマンド解釈
- すべての menu item を初回から palette へ完全収録すること
- file search modal の即時削除
- fuzzy search ranking の過度な最適化

## Decisions

### 1. Palette は provider-based に構成する

palette result は 1 種類ではなく、command provider、workspace file provider、Markdown content provider、recent items provider の複数 source から集約する。

- 採用理由:
  - 既存 search flow を provider として取り込みやすい
  - provider ごとに availability を制御できる
  - 他の AI エージェントが拡張点を理解しやすい
- 代替案:
  - 単一巨大 search 関数: result type ごとの責務が混ざるため不採用

### 2. Result contract は kind と execute payload を持つ

palette result は少なくとも kind、label、secondary text、score、execute payload を持つ。
execute payload は `AppAction` 直接実行で済むものと、file/content navigation のような open-at-location payload を分けて扱う。

- 採用理由:
  - command と navigation result を同じ list に載せやすい
  - 既存 `AppAction` を極力再利用できる
  - result rendering と execution の責務が分かれる
- 代替案:
  - すべて `AppAction` に押し込む: location-aware navigation が不自然になるため不採用

### 3. Empty query state は recent/common entries を出す

command palette は query 入力後だけ役に立つ UI ではなく、開いただけでよく使う command や recent items に辿れる方が価値が高い。
そのため empty query では recent/common entries を表示する。

- 採用理由:
  - keyboard launcher としての価値が上がる
  - 「何ができるか」を発見しやすい
- 代替案:
  - 空入力時は空リスト: palette の初回価値が弱いので不採用

### 4. Existing file-search modal は fallback として残す

現行の file-search modal は既に存在し、即座に削除する必要はない。`v0.15.0` では palette を primary fast path にしつつ、modal は compatibility fallback として残す。

- 採用理由:
  - 段階的移行にしやすい
  - regression risk を抑えられる
  - 他の AI エージェントが既存導線を壊さずに進めやすい
- 代替案:
  - modal の即時撤去: change scope が不必要に広がるため不採用

### 5. Keyboard interaction を first-class とする

palette は modal を開いてからマウスで探す UI ではなく、open → type → arrow / enter で完結できるべきである。
そのため open 時 focus、selection movement、confirm、dismiss の keyboard contract を先に固定する。

- 採用理由:
  - navigation friction を最も下げられる
  - power user value が高い
  - local LLM なしでも体験向上が大きい
- 代替案:
  - mouse-first palette: 既存 UI との差別化が弱くなるため不採用

## Risks / Trade-offs

- [Risk] Provider ごとの ranking が不安定になる -> Mitigation: kind ごとの grouping と predictable ordering contract を先に定義する
- [Risk] Content search 未実装時に palette の仕様が先行しすぎる -> Mitigation: provider availability を optional にし、未実装 provider は非表示にする
- [Risk] 既存 file-search modal と責務が重複する -> Mitigation: palette を primary fast path、modal を compatibility fallback と位置付ける
- [Risk] command 数が増えすぎて palette が noisy になる -> Mitigation: initial command set を common actions に絞る
