## Context

現在の KatanA には Problems Panel と Markdown diagnostics 実装が存在するが、user-facing contract は markdownlint 互換ではない。

- `crates/katana-linter/src/markdown.rs`
  - `MarkdownDiagnostic` は `rule_id`、message、range を持つ
  - ただし rule id は `md-heading-structure` など internal naming であり official code ではない
- `crates/katana-ui/src/app/action.rs`
  - `RefreshDiagnostics` は heading / broken link の 2 rule を直列評価している
- `crates/katana-ui/src/views/panels/problems.rs`
  - Problems Panel は severity icon と line:column、message を表示する
  - rule code、docs link、unsupported state は表示しない
- archived `v0-13-0-markdown-diagnostics-and-problems-panel`
  - app 内 diagnostics と Problems Panel の基本方針は既にある
  - ただし official markdownlint parity までは契約化されていない

ユーザー要望は「公式の No と同期した lint エラー表示」であり、文脈上は表示だけではなく rule behavior parity を求めている。したがって `v0.19.0` では、user-facing に出荷する markdown diagnostics を markdownlint 公式 contract へ寄せ、internal rule 名のまま見せる状態を解消する必要がある。

## Goals / Non-Goals

**Goals:**

- user-facing markdownlint diagnostics について official rule code と挙動を app diagnostics と揃える
- Problems Panel で official code、name、English description、location、docs link を扱えるようにする
- parity 未達の internal rule を user-facing official result と混同させない
- future local LLM autofix がそのまま消費できる diagnostics payload を定義する

**Non-Goals:**

- Node.js runtime や markdownlint CLI を app に同梱すること
- local LLM による autofix 実装
- per-keystroke lint 実行
- markdownlint config file の完全互換実装

## Decisions

### 1. user-facing markdownlint diagnostics は official contract を end-to-end で使う

rule code だけ official に見せて挙動がズレる状態は、ユーザー要望と衝突する。したがって、user-facing に出す diagnostics は official rule code、message、location、severity、docs 参照を一体で扱い、parity が取れていない internal rule を公式互換として出荷しない。

- 採用理由:
  - ユーザー要望の「公式番号と同期」を表示だけで終わらせない
  - 問題パネルと lint 実体のズレを避けられる
- 代替案:
  - rule code だけ official に寄せる: 表示だけ一致して挙動がズレるため不採用
  - internal rule 名を残したまま説明文だけ寄せる: 公式ドキュメントとの往復に失敗するため不採用

### 2. diagnostics payload は official metadata を第一級属性として持つ

future autofix と UI の双方で使うため、diagnostics payload に official rule code、rule title、English description、docs URL を追加する。

- 採用理由:
  - Problems Panel と future AI prompt が同じ情報を共有できる
  - internal rule 名を UI から隠せる
- 代替案:
  - UI 側で別 lookup する: linter と UI で rule source of truth が分裂するため不採用

### 3. parity 検証は fixture ベースで固定する

runtime に Node.js や markdownlint CLI を同梱しなくても、official behavior との整合は fixture corpus と golden test で固定できる。rule ごとに violation / valid case を持ち、false positive と false negative の回帰を防ぐ。

- 採用理由:
  - parity claim をテストで支えられる
  - future autofix 前提の deterministic input を保てる
- 代替案:
  - 手動確認だけで済ませる: rule 追加のたびに回帰しやすいため不採用

### 4. parity 未達の internal rule は hidden または experimental として分離する

parity が取れていない rule を official diagnostics と同列に並べると contract が崩れる。したがって、`v0.19.0` では user-facing default から外すか、明確な experimental 表示を付ける。

- 採用理由:
  - user-facing contract を守れる
  - 実装途中の rule を later release へ安全に送れる
- 代替案:
  - internal rule をそのまま Problems Panel に混在させる: official parity claim を損なうため不採用

### 5. official English message は short description + docs link を基本とする

ユーザー要望では i18n も将来視野にあるが、現時点では official English が現実的である。初期は short English description と docs link を canonical message とし、将来 translation overlay を被せられる形にする。

- 採用理由:
  - 公式との同期軸が明確
  - `v0.25.0` translation overlay の対象として扱いやすい
- 代替案:
  - 独自日本語文言を先に用意する: 公式同期が曖昧になるため不採用

## Risks / Trade-offs

- **[Risk] official behavior parity の検証コストが高い**
  -> Mitigation: user-facing に出荷する rule ごとに fixture corpus と golden test を持つ

- **[Risk] parity 未達 rule を hidden にすると見える rule 数が減る**
  -> Mitigation: release note と docs で experimental / 後続対応予定を明示する

- **[Risk] official docs wording 追随が保守コストになる**
  -> Mitigation: full docs copy ではなく short metadata + docs URL に留める

- **[Risk] current `RefreshDiagnostics` 実装が rule registry 化されず増殖する**
  -> Mitigation: rule catalog と evaluator registry を導入する

## Migration Plan

1. current internal diagnostics rule を棚卸しし、official markdownlint rule へ map する
2. user-facing diagnostics から internal rule 名を排除し、official metadata を payload に追加する
3. rule ごとの violation / valid fixture を追加し、parity test を固定する
4. Problems Panel を official contract ベースの表示へ移行する
5. parity 未達 rule の hidden / experimental 扱いと docs を整備する

## Open Questions

- app 内で docs link を外部ブラウザへ開くか、tooltip / modal に留めるか
- experimental rule を user に見せる設定を用意するか
