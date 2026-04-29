## Context

策定日: 2026-04-25

この change は v1.0.1 の最初の規格として扱う。目的は新機能ではなく、正式リリース後に修正や拡張を安全に続けるための内部整理である。

以下は 2026-04-25 時点の解析結果である。実装着手時には task0 で同日からの差分を確認し、計画を最新状態へ更新する。

現状確認で目立つ事実は次の通り。

- `crates/katana-ui/src/app_action.rs` は 180 行超の single enum に、workspace、document、layout、download、settings、linter、tab group、authoring、image ingest などが同居している。
- `crates/katana-ui/src/app_state.rs` は document、workspace、layout、search、scroll、update、config、diagnostics、command palette、global workspace を一つの root state として直接公開している。
- `crates/katana-ui/src/shell/shell_tests.rs` は 1000 行超、`shell_ui_tests.rs` と `preview_pane/tests.rs` も 1000 行規模で、責務単位の失敗原因が追いにくい。
- `preview_pane`、`views/panels/preview`、`katana-core/src/markdown/*` の間で preview / diagram / render worker の責務境界が曖昧になりやすい。
- integration tests は増えているが、正式リリース後に守るべき product contract、migration contract、error recovery contract が明示的な gate として整理されていない。

## Task 0 差分反映（2026-04-25 14:53 JST）

比較基準は、この change を作成した `512f6864 docs(openspec): v1-0-1-internal-refactoring-test-hardening change を追加` とした。着手時点の `master` は `9ffeb570 feat: 図表backend adapter契約を追加` である。

2026-04-25 の策定後に確認した差分は次の通り。

- `local-llm-ui-integration` は、`v0-23-0-local-llm-lint-autofix` の LLM MVP が `master` に入った後の導線整理として前提が明記された。この v1.0.1 change では LLM UI / Ollama / chat 導線を扱わない。
- `i18n-runtime-safety` Task 1 が `master` に入り、未知の runtime language code は fallback language へ解決されるようになった。v1.0.1 ではこの fallback の再実装を扱わず、settings / state / action の境界整理で既存挙動を壊さないことを検証対象にする。
- `diagram-backend-adapter` Task 1 が `master` に入り、Mermaid / PlantUML 用の backend input、render options、theme snapshot、document context、renderer-neutral output / error、cache key contract が追加された。v1.0.1 の preview / diagram 整理ではこの契約を前提にし、同じ契約型を再定義しない。
- 2026-04-25 の OpenSpec 整理で、無印の `preview-adapter-contract`、`diagram-backend-adapter`、`i18n-runtime-safety`、`local-llm-ui-integration` は superseded archive へ移した。今後は versioned change を正の実施単位とする。
- `.agents` / `.codex` workflow、Makefile、`scripts/runner` には、策定後に v1.0.1 の作業順序を変える差分は確認されなかった。現時点の verification gate は引き続き `make check` を主軸にする。
- `katana-ui` の再計測では、`shell_tests.rs` 1241 行、`shell_ui_tests.rs` 1091 行、`preview_pane/tests.rs` 1025 行が最大の単体テスト群として残っている。integration 側では `tests/integration/preview_pane/tables.rs` 656 行、`preview_pane/diagrams.rs` 273 行が大きい。
- `katana-ui/src` は既に `app/action/*`、`state/*`、`shell/*`、`shell_ui/*`、`views/*`、`widgets/*` に分かれているが、`features/*` という所有境界はまだ存在しない。Task 1 ではこの現状を前提に、単純移動で済む領域と service boundary の再設計が必要な領域を分ける。

この反映により、Task 1 以降の優先順は次のように最適化する。

1. Task 1 では、master に入った i18n fallback と diagram backend contract を既存前提として棚卸し対象へ含めるが、再実装対象にはしない。
2. Task 2 のディレクトリ再設計は、`features/*` 新設を確定する前に、既存 `app/action/*` と `state/*` の所有境界を表にする。
3. Task 3 は、引き続き `AppAction`、root `AppState`、shell dispatch を優先する。preview / diagram の深い実装移行は active change との衝突を避ける。
4. Task 4 は、巨大 unit test の分割に加え、既存 integration tests の大きい preview table / diagram 領域を release regression gate へどう接続するかを明記する。
5. Task 5 は、現在の `make check` が macOS impacted test、Linux workspace test、Windows xwin check を含む前提で、常時 gate と release-only gate の境界を再判断する。

## Task 1 構造棚卸し（2026-04-25 15:28 JST）

Task 1 では実装移動を行わず、v1.0.1 で扱う整理対象と、active change または後続 version へ送る対象を分けた。行数は 2026-04-25 15:28 JST 時点の `master` で再計測した。

### 大きいモジュール / テストファイル

| 対象 | 観測結果 | 分類 | 先に必要な契約テスト / 確認 |
| --- | --- | --- | --- |
| `crates/katana-ui/src/shell/shell_tests.rs` | 1241 行。shell lifecycle、download、update、workspace、action dispatch 系の期待が同居している。 | 機械的分割を先行可能 | 分割前後で shell action contract の test name と assertion を対応付ける。 |
| `crates/katana-ui/src/shell_ui/shell_ui_tests.rs` | 1091 行。shell UI layout と操作導線が同居している。 | 機械的分割を先行可能 | layout contract、toolbar contract、panel visibility contract を分けて同じ観点を維持する。 |
| `crates/katana-ui/src/preview_pane/tests.rs` | 1025 行。preview 表示、code block、diagram、scroll / anchor 系が混在している。 | 境界再設計後に分割 | `v0-28-0-preview-adapter-migration` の metadata / action contract を前提に、semantic assertion を先に固定する。 |
| `crates/katana-ui/tests/integration/preview_pane/tables.rs` | 656 行。table 表示の integration regression が大きい。 | release gate 分類 | 通常 integration contract に残す範囲と release-only gate に回す範囲を決める。 |
| `crates/katana-ui/src/svg_loader/logic.rs` | 590 行。SVG decode、cache、theme adaptation の責務が寄りやすい。 | 境界再設計 | loader input / output と cache failure の contract test を先に定義する。 |
| `crates/katana-ui/src/main.rs` | 403 行。bootstrap と runtime wiring が集まっている。 | 境界再設計 | entrypoint から config / service assembly を分離する前に startup smoke を固定する。 |
| `crates/katana-platform/src/cache/default/mod.rs` | 412 行。default cache backend の実装と policy が同居している。 | 境界再設計 | cache migration、read/write、invalid data recovery の contract test を確認する。 |
| `crates/katana-platform/src/filesystem/scanner.rs` | 347 行。filesystem traversal と workspace scan policy が近い。 | 境界再設計 | hidden file、ignore、symlink、permission error の期待を固定する。 |
| `crates/katana-platform/src/theme/builder.rs` | 322 行。theme assembly が大きく、settings との接続も近い。 | 境界再設計 | theme token の fallback と serialization roundtrip を固定する。 |
| `crates/katana-core/src/system/process.rs` | 298 行。外部 process 実行の共通面を持つ。 | 境界再設計 | process success / failure / timeout / stderr handling の contract を定義する。 |
| `crates/katana-core/src/markdown/mermaid_renderer/render.rs` | 232 行。Mermaid 外部 renderer 実装。 | 後続送り | `v0-31-0-native-diagram-renderer-backends` 後続 task で adapter implementation に移す。 |
| `crates/katana-core/src/preview/section/mod.rs` | 166 行。preview section metadata。 | 後続送り | `v0-28-0-preview-adapter-migration` の metadata contract と重複させない。 |

### 責務別の現状分類

| 領域 | 現状責務 | v1.0.1 での扱い | 先に固定するもの |
| --- | --- | --- | --- |
| `AppAction` | document、workspace、layout、settings、diagnostics、tab、download、update、authoring、image ingest が single enum に集まる。 | 境界再設計 | 領域 action へ分ける前に、既存 action から状態変化までの contract test を作る。 |
| `AppState` | document、workspace、layout、search、scroll、update、config、diagnostics、command palette、global workspace を root state が直接持つ。 | 境界再設計 | feature state の query / command API を追加する前に不変条件を列挙する。 |
| shell dispatch | `app/action/*` は分割済みだが、2749 行規模で root dispatch への依存が残る。 | 境界再設計 | root routing と領域 handler の責務表を作る。 |
| preview rendering | `preview_pane`、`views/panels/preview`、`katana-core/src/preview`、markdown renderer の境界が交差する。 | 後続送りを含む | adapter metadata / action contract を先に入れ、v1.0.1 では重複実装しない。 |
| diagnostics | linter diagnostics、fix、autofix entry が action / view / state にまたがる。 | 境界再設計 | diagnostic refresh と fix application の入力 / 出力 contract を定義する。 |
| workspace | workspace tree、history、global workspace、file scan、session restore が複数層にまたがる。 | 境界再設計 | workspace repository と UI state の同期条件を固定する。 |
| settings | settings UI、persistence、theme / language / editor options が接続している。 | 境界再設計 | settings migration、fallback、UI 反映の contract test を固定する。 |

### 範囲から除外する既存実装

| 対象 | 状態 | v1.0.1 での扱い |
| --- | --- | --- |
| i18n fallback | 無印 `i18n-runtime-safety` Task 1 として `185d2913` で `master` に実装済み。無印 change は archive 済み。 | 再実装しない。`v0-30-0-advanced-i18n-runtime` で後続を扱う。 |
| diagram backend contract | 無印 `diagram-backend-adapter` Task 1 として `9ffeb570` で `master` に実装済み。無印 change は archive 済み。 | 契約型を再定義しない。`v0-31-0-native-diagram-renderer-backends` で後続を扱う。 |
| local LLM UI 前提整理 | 無印 `local-llm-ui-integration` は未実装の参考資料として archive 済み。 | v1.0.1 では LLM chat / Ollama / model selection を扱わない。必要時は local LLM 系 versioned change で再起票する。 |
| preview adapter 契約 | 無印 `preview-adapter-contract` の initial DTO / contract は `05341608` で `master` に実装済み。無印 change は archive 済み。 | `v0-28-0-preview-adapter-migration` の既存前提として扱い、preview 深部移行は v1.0.1 に吸収しない。 |

この分類により、v1.0.1 の最初の実装対象は `AppAction`、root dispatch、root `AppState`、巨大 test file の contract 分割、release regression gate に絞る。preview / diagram / LLM / i18n runtime fallback は、versioned change または実装済み差分を尊重し、同じ契約や UI を再作成しない。

## Goals / Non-Goals

**Goals:**

- 着手時に 2026-04-25 からの差分を反映し、計画を更新する。
- ディレクトリ整理だけで済む変更と、実装責務の再設計が必要な変更を分ける。
- `katana-ui` の action / state / service / view の境界を再定義する。
- 大きい test file を feature contract 単位へ分割する。
- release regression gate を `make check` または専用 target の内訳として説明できる状態にする。
- 挙動維持の refactor を原則にし、各段階で既存機能の回帰を検知する。

**Non-Goals:**

- v1.0.1 で UI を大きく変えること。
- preview adapter、diagram backend、local LLM など個別機能 change の実装をこの change に混ぜること。
- test を通すためだけに production code を歪めること。
- visual snapshot test を新規導入すること。

## Decisions

### 1. 着手前に策定日からの差分を反映する

2026-04-25 時点の解析結果だけを固定計画として扱わない。実装開始時に `master`、active OpenSpec、`katana-ui` の構造、test runner、Makefile を再確認し、増減した機能や移動済み module を task に反映する。

### 2. ディレクトリ再設計と内部実装再設計を分ける

ファイル移動だけで責務が明確になるものは「機械的な移動」として小さく扱う。一方、`AppAction`、root `AppState`、shell dispatch、preview rendering、document mutation のように境界そのものが曖昧なものは、service boundary と contract test を先に定義してから移す。

### 3. `katana-ui` を feature module と shell module に分ける

目標構造は次の方向にする。

- `features/document`: document open/save/edit/dirty state
- `features/workspace`: workspace tree/history/global workspace
- `features/preview`: preview state、render metadata、render worker integration
- `features/diagnostics`: linter diagnostics、fix、autofix entry
- `features/settings`: settings UI と persistence adapter
- `features/search`: document / workspace search
- `shell`: app lifecycle、top-level dispatch、native menu、frame composition
- `views`: 表示専用の egui surface
- `widgets`: 汎用 UI 部品

この構造は一度に作り替えず、feature ごとに移行する。

### 4. `AppAction` は領域ごとの action に分割する

root action enum は top-level routing のみを担う。document、workspace、layout、settings、diagnostics、preview、tab などは領域 action に分割し、dispatch も領域 handler に寄せる。これにより無関係な action が同じ巨大 match に集まる状態を避ける。

### 5. Root `AppState` は公開 mutable bag から feature state composition へ寄せる

root state は各 feature state を持つが、直接 mutable field を触る範囲を減らす。feature state には小さい query / command method を用意し、document mutation、workspace mutation、layout mutation の invariants を内部で守る。

### 6. Test は contract と harness を分ける

巨大 test file を単に小分けにするだけではなく、何を守るテストかを分類する。

- unit contract: pure logic、state transition、adapter behavior
- integration contract: user action から state / rendered semantic result まで
- regression contract: 過去 bug の再発検知
- release gate: v1.0.0 後に壊してはいけない product workflow

Visual snapshot に頼らず、semantic assertions、state assertions、fixture output assertions を主に使う。

## Risks / Trade-offs

- [Risk] 策定日から実装着手までに構造が変わる → task0 で差分確認と計画更新を必須にする。
- [Risk] 大規模移動で conflict が増える → 機械的な移動と behavior change を別 task / 別 PR に分ける。
- [Risk] 境界を増やしすぎて実装が重くなる → feature boundary は現在の実利用単位から始める。
- [Risk] Test 分割だけで検知力が上がらない → release gate の対象 workflow と assertion を明文化する。
- [Risk] Refactor 中にユーザー向け挙動が変わる → 各 task の完了条件に behavior-preserving verification を含める。

## Migration Plan

1. 実装着手時に 2026-04-25 からの差分を確認し、計画を最新化する。
2. 現状構造の inventory を作り、機械的な移動と境界再設計を分ける。
3. `AppAction` と dispatch を領域 action へ分割する準備を行う。
4. document / workspace / preview / diagnostics の順に feature boundary を固める。
5. 巨大 test file を contract 単位に分割する。
6. release regression gate を Makefile / runner から実行できる形にする。
7. すべての段階で `make check` と対象 contract test を通す。

## Open Questions

- feature module を `crates/katana-ui/src/features/*` に新設するか、既存 `app` / `state` / `views` の下で段階移行するか。
- release gate を `make check` に常時含めるか、`make release-check` として分けるか。
- integration test の serial / parallel 分割基準を現在の runner に合わせて再設計するか。
