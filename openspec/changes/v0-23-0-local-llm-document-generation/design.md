## Context

`v0.22.0` で local LLM provider を user が設定・選択できる前提が整うと、次はその provider を文書生成へ広げる必要がある。現行 KatanA には次の土台がある。

- active document buffer を保持し、save / dirty state を扱える
- workspace file を開く、保存する、作成する modal 基盤がある
- command inventory と shortcut 拡張の土台がある

一方で、generation の出力先を current document / new file / template scaffold の 3 系統で統一的に扱う job model はまだ存在しない。ユーザー要望では 3 つを同時に実装したいので、`v0.23.0` では出力 target ごとの個別実装ではなく、共通の generation contract を先に定義する。

## Goals / Non-Goals

**Goals:**

- current document への挿入、新規 Markdown file 生成、template-based scaffolding を同一 release で提供する
- generation 前に input context と output target を明示する
- write 前に user が内容を確認できるようにする
- apply 後に editor、workspace、dirty state が正しく更新されるようにする

**Non-Goals:**

- remote AI provider のサポート拡張
- multi-step autonomous writing agent
- translation overlay の実装
- repository 全体を自動収集して無制限に prompt へ流すこと

## Decisions

### 1. 3 つの出力先は 1 つの generation job model で表現する

current document insert、new file、template scaffold を別々の UI 機能として分岐させるのではなく、共通の request shape に `target kind` を持たせて処理する。

- 採用理由:
  - provider 呼び出しと preview UI を共通化できる
  - 将来 output target を増やしても拡張しやすい
- 代替案:
  - 出力先ごとに別 service を作る: generation quality と確認導線が分裂するため不採用

### 2. 生成結果は write 前に必ず review 可能にする

current document への挿入でも new file 作成でも、生成結果をそのまま書き込まず、preview または差分確認を経てから適用する。

- 採用理由:
  - 意図しない上書きや挿入位置の誤りを防げる
  - template scaffold でも file 作成事故を抑えられる
- 代替案:
  - 即時書き込みする: user control が弱く不採用

### 3. generation context は active scope を明示して収集する

prompt 入力に使う context は、active document、selection、workspace root、user prompt などの明示的な範囲に限定する。

- 採用理由:
  - local model でも処理量を抑えやすい
  - 何を送ったか user に説明しやすい
- 代替案:
  - workspace 全体を自動収集する: token / latency / privacy の面で不採用

### 4. template scaffold は preset と destination を分けて扱う

template generation は単なる文面生成ではなく、preset、destination path、ファイル名を伴う。したがって prompt と output path を分離して扱う。

- 採用理由:
  - template 系の再利用性が高い
  - file ops と結合しやすい
- 代替案:
  - free-form prompt だけで template も表現する: destination や構造が曖昧になるため不採用

## Risks / Trade-offs

- **[Risk] 3 出力先を同時に入れることで UI が複雑になる**  
  -> Mitigation: 共通 job model と preview UI を使い、entry point だけ分ける
- **[Risk] current document insert が既存内容を壊す**  
  -> Mitigation: insert position と preview を明示し、undo 可能な apply にする
- **[Risk] template scaffold が file collision を起こす**  
  -> Mitigation: destination 確認と overwrite policy を追加する

## Migration Plan

1. generation job model と target kinds を定義する
2. current document / new file / template scaffold の context builder を実装する
3. generation preview と apply flow を共通化する
4. editor / workspace / file ops に各 entry point を追加する
5. write 後の refresh、dirty state、error recovery を統合する

## Open Questions

- template scaffold の preset を固定テンプレートから始めるか、user-defined template も初期から許可するか
- current document insert の既定位置を cursor、selection 置換、末尾追記のどれにするか
