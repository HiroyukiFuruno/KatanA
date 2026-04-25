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
- active OpenSpec には `preview-adapter-contract`、`diagram-backend-adapter`、`i18n-runtime-safety`、`local-llm-ui-integration` と、v0.22.x から v0.31.0 系の複数 change が残っている。v1.0.1 はそれらの機能 change を吸収せず、内部構造と回帰検知の土台に限定する。
- `.agents` / `.codex` workflow、Makefile、`scripts/runner` には、策定後に v1.0.1 の作業順序を変える差分は確認されなかった。現時点の verification gate は引き続き `make check` を主軸にする。
- `katana-ui` の再計測では、`shell_tests.rs` 1241 行、`shell_ui_tests.rs` 1091 行、`preview_pane/tests.rs` 1025 行が最大の単体テスト群として残っている。integration 側では `tests/integration/preview_pane/tables.rs` 656 行、`preview_pane/diagrams.rs` 273 行が大きい。
- `katana-ui/src` は既に `app/action/*`、`state/*`、`shell/*`、`shell_ui/*`、`views/*`、`widgets/*` に分かれているが、`features/*` という所有境界はまだ存在しない。Task 1 ではこの現状を前提に、単純移動で済む領域と service boundary の再設計が必要な領域を分ける。

この反映により、Task 1 以降の優先順は次のように最適化する。

1. Task 1 では、master に入った i18n fallback と diagram backend contract を既存前提として棚卸し対象へ含めるが、再実装対象にはしない。
2. Task 2 のディレクトリ再設計は、`features/*` 新設を確定する前に、既存 `app/action/*` と `state/*` の所有境界を表にする。
3. Task 3 は、引き続き `AppAction`、root `AppState`、shell dispatch を優先する。preview / diagram の深い実装移行は active change との衝突を避ける。
4. Task 4 は、巨大 unit test の分割に加え、既存 integration tests の大きい preview table / diagram 領域を release regression gate へどう接続するかを明記する。
5. Task 5 は、現在の `make check` が macOS impacted test、Linux workspace test、Windows xwin check を含む前提で、常時 gate と release-only gate の境界を再判断する。

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

0. 実装着手時に 2026-04-25 からの差分を確認し、計画を最新化する。
1. 現状構造の inventory を作り、機械的な移動と境界再設計を分ける。
2. `AppAction` と dispatch を領域 action へ分割する準備を行う。
3. document / workspace / preview / diagnostics の順に feature boundary を固める。
4. 巨大 test file を contract 単位に分割する。
5. release regression gate を Makefile / runner から実行できる形にする。
6. すべての段階で `make check` と対象 contract test を通す。

## Open Questions

- feature module を `crates/katana-ui/src/features/*` に新設するか、既存 `app` / `state` / `views` の下で段階移行するか。
- release gate を `make check` に常時含めるか、`make release-check` として分けるか。
- integration test の serial / parallel 分割基準を現在の runner に合わせて再設計するか。
