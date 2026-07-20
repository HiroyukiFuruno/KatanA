## Superseded Architecture Notice

The current ledger immediately below and the canonical
`openspec/specs/html-file-preview/spec.md` are the v0.22.33 release sources of
truth. Sections after the historical-evidence marker preserve pre-2026-07-17
history only. The current viewer is KRR's in-process Rust/V8 runtime;
Chromium, Chrome for Testing, WebView, external helpers, downloads, and
browser runtime assets are prohibited. KRR implementation detail is tracked in
`openspec/changes/v0-4-0-html-dom-runtime/tasks.md`.

## Current Rust/V8 Release Ledger (2026-07-20)

- [x] KRR `0.4.2` is published and verified from crates.io. The interactive
  runtime is in-process Rust/V8 and packages no Chromium, WebView, helper, or
  browser archive.
- [x] KRR `0.4.3` is published and verified after its full release gate. It fixes the
  Japanese missing-glyph frame with browser-style system-font fallback while
  retaining the unchanged 10 MiB crate gate.
- [x] KDV `0.3.0` is published and verified as a worker-backed adapter over the
  KRR `0.4.x` registry line.
- [x] KDV `0.3.1` is published and verified after its full release gate. It coalesces
  adjacent scroll/resize commands without crossing pointer or navigation
  command boundaries.
- [x] KatanA and its headless harness resolve KDV `0.3.1` and KRR `0.4.3`
  from crates.io with checksums; no path/git override exists for either
  dependency.
- [x] KatanA declares KDV `0.3.1` / KRR `0.4.3` as its minimum registry
  requirements; the release guard rejects stale `0.3.0` / `0.4.0` manifest
  requirements independently of the lockfiles.
- [x] KatanA presents complete KRR frames at physical-pixel viewport size and
  forwards pointer, text, focus, scroll, resize, and runtime-confirmed
  navigation without parsing HTML or performing hit-testing.
- [x] Pointer capture is released even when pointer-up occurs outside the HTML
  viewport, focus loss is forwarded when interaction moves outside the surface,
  and logical pointer coordinates are not confused with physical frame pixels.
- [x] Top-level navigation reuses the tab's existing KDV adapter, moves the
  browser surface to the target document path, and retains initial, fragment,
  and target origins in KatanA-owned tab history.
- [x] HTML save/watcher refresh reads the latest saved source after one
  coalescing delay, and watcher events remain queued while another app action
  is pending instead of being consumed and lost.
- [x] Source-construction failures remain typed viewer errors and do not start
  an empty fallback session. Invalid final redirect origins are rejected.
- [x] Headless acceptance passes all 60 steps with external `style.css` and
  `actions.js`, accordion, button mutation, input event, prevented navigation,
  down/up viewport scrolling, allowed local link navigation, reload, resize,
  active-document assertions, semantic RGB assertions, and positive
  frame-difference assertions. Complete same-document/external fragment
  origins, raw KRR frame pixels, and composed screenshot pixels are asserted
  independently. Committed Japanese input is visible without repeated
  missing-glyph boxes in the input, V8 result, and status regions. RGB click
  targets are clipped to the current HTML surface in physical pixels instead
  of relying on fixed whole-window coordinates.
- [x] Targeted tests, OpenSpec 54/54, workspace check, strict clippy, AST lint,
  fixture integration, screenshot runner 14/14, and the unchanged strict
  meaningful-line coverage gate pass.
- [x] Release contract regression tests independently reject stale KDV `0.3.0`
  and KRR `0.4.2` locks. Production preflight and pre-PR readiness therefore
  accept only the current registry-only patch floors.
- [x] Publish and verify KRR `0.4.3`, then KDV `0.3.1`; rebuild KatanA from
  registry-only KDV/KRR dependencies and rerun the 60-step headless
  acceptance.
- [x] User explicitly approved the current headless screenshots on 2026-07-21
  before any
  KatanA commit, push, PR update, merge, publish, or `v0.22.33` release.
- [x] The final post-review release candidate reran all 60 steps after
  persistent-session/history and refresh-coalescing fixes; all 11 PNGs matched
  the approved evidence with `AE=0`, `RMSE=0`, and unchanged SHA-256 values.

Everything below this ledger is retained as historical task/recovery evidence
and is not the current release source of truth.

## Definition of Ready (DoR)

- [x] v0.22.33 は static preview を許可せず、KRR の in-process Rust/V8 session を HTML/CSS/JavaScript/form/hit-test/navigation の唯一の source of truth とする。
- [x] KatanA は主文書の raw HTML と完全な document URL を供給し、KDV は KRR browser-session adapter のみを担う責務境界が OpenSpec に固定されている。
- [x] browser runtime failure 時は typed error とし、static parser、direct HTML normalizer、Markdown renderer、export image へ fallback しない。
- [x] 公開済み KRR `0.4.3` -> KDV `0.3.1` -> KatanA `v0.22.33` を直列に統合し、KDV/KRR の local path/git dependency を使わない。
- [x] headless 60-step evidence が CSS layout、accordion、JavaScript action、text/IME input、fragment/link navigation、reload、resize、complete origin、raw/composed frame、文字の非重なりを示し、ユーザーが明示 OK している。

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-33-html-file-preview` またはリリース用統合ブランチ（例: `release/v0.22.33`）
- **作業ブランチ**: 標準は `v0-22-33-html-file-preview-task-x`、リリース用は `feature/v0.22.33-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. Repository Boundary Gate

### Definition of Done (DoD)

- [x] KDV `0.2.x` direct HTML normalizer/static surface は interactive viewer contract を満たさず、KDV `0.3.0` browser-session adapter が必要と確認する。
- [x] KRR `0.4.x` が in-process Rust/V8 session、complete frame、input、navigation、resource policy を所有し、external browser runtime を持たないことを確認する。
- [x] KatanA/KDV に HTML semantics を補完せず、KRR -> KDV -> KatanA の公開順序と ownership を OpenSpec に固定する。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 1.1 KDV `0.2.x` の direct HTML normalizer / preview surface と KRR `0.4.0` browser API を棚卸しする。
- [x] 1.2 KatanA は source host/native surface、KDV は worker-backed adapter、KRR は browser semantics という判断表を確定する。
- [x] 1.3 KDV `0.3.0` adapter の入力、frame/input/navigation/error 出力、static fallback 禁止を KRR OpenSpec と同期する。
- [x] 1.4 KRR `0.4.0`、KDV `0.3.0`、KatanA `v0.22.33` の公開順序を release block として記録する。

### Repository Boundary Decision

| repo | 確認結果 | 判断 |
| --- | --- | --- |
| `katana-document-viewer` | `0.2.x` direct HTML normalizer/static preview surface は CSS/JS/form/navigation/browser hit-test を持たず不合格。 | 公開済み KRR `^0.4.0` session を worker 上で所有し、latest complete frame、input、navigation、typed error だけを中継する `0.3.0` adapter を公開する。parser/CSS/static image fallback は interactive path に持たない。 |
| `katana-render-runtime` | `0.4.3` candidate は in-process Rust/V8 HTML/CSS/JS/input/navigation/resource policy、complete frame、font fallback、fragment reflow を実装し strict release gate/coverage を通過済み。 | HTML semantics の唯一の owner。Chromium/WebView/helper/archive を packaging せず、`0.4.3` 公開確認後に KDV patch を公開する。 |
| `KatanA` | 現行 diff は KDV static image surface を hover-only image として表示するため browser interaction を通せない。 | KDV `0.3.0` 公開後に browser session host へ置換し、raw HTML + complete URL、input、navigation/history、reload、packaged runtime を接続する。 |

## 2. Workspace and File Open Contract

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [x] `.html` / `.htm` が standard visible extensions に含まれる。
- [x] file open dialog と drag-and-drop が `.html` / `.htm` を openable file として扱う。
- [x] 既存の workspace filtering は HTML file にも通常どおり適用される。
- [x] settings 由来の visible extensions / 新規ファイル作成候補でも `.html` / `.htm` が標準表示対象から漏れない。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 2.1 `TreeEntry::standard_visible_extensions()` に HTML 拡張子を追加する。
- [x] 2.2 `FileOpenOps::supported_extensions()` / dialog extension / dropped file 判定のテストを追加する。
- [x] 2.3 workspace filter 有効時に HTML visibility が filter を迂回しないことを確認する。
- [x] 2.4 `settings.workspace.visible_extensions` が空または旧設定でも、workspace scan は実効 standard visible extensions を補完し、新規ファイル候補は `.html` / `.htm` を補完する。

## 3. Browser Session Preview Routing

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [x] `.html` / `.htm` / URL active document は KDV `0.3.1` browser-session adapter へ流れる。
- [x] KatanA は raw HTML と complete document URL を渡し、latest complete frame を exact viewport に表示する。
- [x] pointer、keyboard、text/IME、focus、scroll、resize、navigation/history、reload が KRR session へ end-to-end で接続される。
- [x] WebView、HTML parser、CSS cascade、JavaScript interpreter、browser hit-test、static fallback が KatanA/KDV interactive path に存在しない。

- [x] 3.1 active document path から HTML document を識別する小さな helper を追加する。
- [x] 3.2 `refresh_preview` / `full_refresh_preview` の static `PreviewSurfaceImage` 経路を KDV browser-session lifecycle へ置換する。
- [x] 3.3 metadata/style/script、responsive table、link、accordion、JavaScript button、form input、reload を含む browser acceptance fixture を接続する。
- [x] 3.4 integration test で complete frame、raw input forwarding、browser-confirmed navigation、typed runtime error、static fallback 禁止を固定する。

## 4. Markdown-only Tool Isolation

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [x] Markdown diagnostics は HTML file を lint 対象にしない。
- [x] Markdown formatting は HTML file を formatter へ渡さない。
- [x] HTML file preview は Markdown export adapter を preview rendering に使わない。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 4.1 `process_diagnostics` の Markdown extension gate が HTML file を除外するテストを追加する。
- [x] 4.2 Markdown formatting path gate が HTML file を拒否する既存挙動をテストで固定する。
- [x] 4.3 export panel / export action が HTML active document で誤解を招く場合、表示制御または明示エラーを設計に沿って追加する。

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [/] 5.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする。
  - KRR packaged Chromium evidence: `/Users/hiroyuki_furuno/works/private/katana-render-runtime/tmp/release-smoke-final.ev5cwM/html-browser-contact-sheet-final-bordered.png`
  - KRR result: initial、accordion click、JavaScript DOM mutation、text input の4状態を完全な960x720 frameとして確認できる。これはKRR engine単体の合格材料であり、KatanA end-to-endの合格材料ではない。
  - KatanA rejection evidence: `tmp/v0-22-33-html-file-preview-evidence/01-html-preview.png`
  - KatanA result: style/script相当が本文表示され、static surfaceで操作前後を証明できない。release approvalには使用しない。
  - Native runner repair evidence: `tmp/v0-22-33-native-runner-evidence-active-assert/02-html-preview-native-window.png`。固定viewport、fixture workspace復元、AccessKitによるfile open、active document assertion、native captureが完走する。
  - Final acceptance rejection evidence: `tmp/v0-22-33-html-final-acceptance-current-rerun/01-initial-render.png`。accordion、JavaScript button、text input、link navigationを含むfixtureでCSS sourceが本文表示された。accordion click後の`02-accordion-open.png`はinitialと同じSHA-256 `35f7614ff363ce27373f4d7843aca39fbe7a78ac8c207fc71fb6b9037feffd2f`で、changed pixelsが`0`のためgateが停止した。
  - Semantic rejection evidence: `tmp/v0-22-33-html-final-acceptance-semantic-rejection/01-initial-render.png`。fixture固有の初期状態色`rgb(230,245,239)`が`0 px / required 500 px`で、static source表示を操作前に拒否した。
  - Evidence harness contract: system temp workspaceの復元禁止を回避するoutput-local fixture、PID固定のnative操作、AccessKit file open、`click_at`、`type_text`、`refresh_document`、`resize_window`、active document assertion、minimum changed-pixel assertion、fixture固有のminimum semantic RGB-pixel assertionを実装した。画面スクロールやrepaintだけの差分では合格できない。
  - Semantic gate verification: `cargo test --manifest-path scripts/screenshot/Cargo.toml`は`14/14`成功。`bash scripts/release/test-html-browser-release-contract.sh`は成功し、button状態のRGB assertion欠落とHTML clickへの固定search boundsを拒否する回帰を含む。
  - Current headless evidence: `tmp/v0-22-33-html-headless-registry-20260721-final/` は公開済み KDV `0.3.1` / KRR `0.4.3` の registry-only lock で60/60成功。accordion、button、committed Japanese IME、prevented navigation、scroll、same-document/external fragment、reload、resizeを含み、complete origin、raw KRR frame、composed screenshotを独立検査した。2026-07-21 にユーザー承認済み。
- [/] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）。
  - 2026-07-17 root feedback: 個別の文字重なり補修ではなく、HTML として評価できない static preview 契約そのものを撤回する。canonical/archived spec、proposal、design、release evidence、CHANGELOG、release gate を browser-equivalent contract へ同期済み。実装・packaged evidence は upstream publication 待ち。
- [/] 5.3 2026-07-04 のリカバリー指示: v0.29.0 は本来 v0.22.33 として扱うべき内容だったため取り下げを優先する。v0.22.33 はユーザーの明示 OK が出るまでリリースしない。
  - 現状確認: GitHub Release `v0.29.0` は公開済み、GitHub Release `v0.22.33` は未作成。
  - 2026-07-04 再確認: `gh release view v0.29.0` は公開済み release を返す。`gh release view v0.22.33` は `release not found`。remote tag は `v0.29.0` のみ存在し、`v0.22.33` は未作成。
  - 2026-07-04 リカバリー実施: `gh release delete v0.29.0 --cleanup-tag --yes` を実行し、誤 release と remote tag を削除した。
  - 2026-07-04 リカバリー検証: `gh release view v0.29.0` は `release not found`。`git ls-remote --tags origin v0.29.0 v0.22.33` は空。`gh release view v0.22.33` は `release not found`。つまり誤 release/tag は取り下げ済みで、正規 release は未作成のまま。
  - 2026-07-06 再検証: KatanA GitHub Release `v0.29.0` / `v0.22.33` はどちらも `release not found`。remote tag `v0.29.0` / `v0.22.33` も存在しない。
- [/] 5.4 2026-07-04 の追加指示: 今回のような飛び版を防ぐため、v0.22.33 release check は最新の有効な GitHub Release から `patch +1` のみ許可する。
  - 対応: `scripts/release/check-pr-ready.sh` から `scripts/release/check-version-increment.sh` を呼び出し、GitHub の最新公開 release、`CHANGELOG.md` の target release、直前 release heading を比較する。現在の最新公開 `v0.22.32` からは `v0.22.33` だけを許可し、取り下げ済み `v0.29.0`、minor/major update、偽装した changelog base を拒否する。
  - Verification:
    - `bash -n scripts/release/check-version-increment.sh`
    - `bash scripts/release/test-version-increment.sh`
    - `./scripts/release/check-version-increment.sh 0.22.33` -> live GitHub Release `v0.22.32 -> v0.22.33` を許可
- [/] 5.5 2026-07-17 の HTML viewer 契約是正: v0.22.33 は browser-equivalent HTML session を必須とし、静的 preview、JS/navigation/form の defer、static fallback を release scope として認めない。
  - Root cause: canonical/archived OpenSpec が CSS-aware static preview を合格条件にしていたため、KDV static parser/image surface と KatanA hover-only image path が仕様上許容され、HTML として評価できない画面が作られた。
  - Corrected boundary: KatanA は主文書の raw HTML + complete document URL と native surface/history を所有し、KDV `0.3.x` は worker-backed adapter、KRR `0.4.x` in-process Rust/V8 session は HTML/CSS/JS/event/form/hit-test/navigation/resource policy を所有する。
  - Acceptance: CSS layout、accordion、JavaScript button、text/IME input、fragment/link navigation、reload、resize、complete origin、raw/composed frame、文字の非重なりを deterministic headless harness で示す。runtime failure は typed error とし static parser/export image へ fallback しない。
  - Current status: KRR `0.4.3` / KDV `0.3.1` は公開確認済み。KatanA の registry-only 60-step headless acceptance とユーザー承認も完了した。
  - Superseded decision: 2026-07-06 の「KDV `0.2.8` CSS-aware static preview を v0.22.33 scope とし JS/navigation/forms を defer」は撤回し、acceptance evidence に使用しない。
- [/] 5.6 2026-07-04 の settings 指摘: `.html` / `.htm` は tree に表示するファイルとして許容し、settings 由来の visible extensions / 新規ファイル候補から漏れないようにする。
  - 対応: `effective_visible_extensions()` で workspace scan / refresh が standard visible extensions を補完し、`file_creation_visible_extensions()` で new file modal が `.html` / `.htm` を補完する。
  - Verification:
    - `cargo test -p katana-ui visible_extensions_include -- --nocapture`
    - `cargo test -p katana-ui handle_open_explorer_includes_html_files_without_user_visible_toggle -- --nocapture`
    - `cargo test -p katana-ui workspace -- --nocapture`
    - `cargo test -p katana-ui supported_extensions_include_html_documents -- --nocapture`
    - `cargo test -p katana-ui openable_files_accept_html_and_htm_paths -- --nocapture`
- [/] 5.7 published browser chain 指摘: KRR `0.4.3` -> KDV `0.3.1` -> KatanA `v0.22.33` の patch fixes を順番に公開確認し、KatanA は crates.io dependency だけを利用する。
  - Required sequence: KRR `0.4.3` の crates.io 公開確認後に KDV `0.3.1` を公開する。両方の公開確認後にだけ KatanA lockfile を registry-only で更新し、60-step headless evidence を作り直す。
  - Current status: KRR `0.4.3`、KDV `0.3.1` の順で GitHub Release / crates.io 公開を確認した。KatanA と headless harness は checksummed crates.io package のみを解決し、production preflight と 60-step registry-only acceptance が成功した。
  - Prohibited: KDV/KRR の local path/git dependency、KDV static candidate、Chromium/WebView/external helper/browser archive を最終 package に使わない。
  - Superseded historical verification (static KDV `0.2.8` candidate; release acceptance には使用しない):
    - Katana `cargo metadata --format-version 1 --locked --no-deps` shows `katana-document-viewer` and `katana-render-runtime` dependencies resolving from `registry+https://github.com/rust-lang/crates.io-index`.
    - Katana `[patch.crates-io]` contains only existing vendor patches for `egui_commonmark`, `egui_commonmark_backend`, `egui-winit`, and `mathjax_svg`; it does not patch KDV or KRR.
    - KDV `cargo metadata --format-version 1 --locked --no-deps` shows KDV `0.2.8` depending on `katana-render-runtime` from crates.io registry, not a local path.
  - 2026-07-04 KDV `0.2.8` readiness evidence:
    - Clean verification was performed on branch `feature/html-css-preview-v0.22.33-clean` in isolated worktree `/Users/hiroyuki_furuno/works/private/katana-document-viewer-html-css-isolated` because the original KDV worktree was being rewritten by an external Zed/Claude process with stale KUC-based changes.
    - Boundary scan found no KDV core dependency on `katana-ui-core`, `egui`, `winit`, or `vello`; the only match was the dependency-guard test fixture text.
    - `just VERSION=0.2.8 release-target-check` passed.
    - `release_publish_depends_on_release_check` now covers that `release-check` depends on `release-target-check`, `release-target-check` calls `verify-release-target.py`, `v0.2.7 -> v0.2.8` is allowed, and `v0.2.7 -> v0.29.0` is rejected.
    - `python3 scripts/release/assert-viewer-recovery-dod.py --self-test` passed.
    - `bash scripts/release/assert-crates-not-published.sh 0.2.8` passed and confirmed `katana-document-viewer 0.2.8 is unpublished`.
    - `python3 scripts/release/verify-release-target.py --target-version v0.29.0 --latest-version v0.2.7` rejected the skipped release line as expected.
    - `env RTK="rtk proxy" CARGO_INCREMENTAL=0 just storybook-release-acceptance-artifacts` followed by `env RTK="rtk proxy" CARGO_INCREMENTAL=0 KDV_RELEASE_DOD_SKIP_ACCEPTANCE_FRESHNESS=1 just VERSION=0.2.8 release-check` passed.
    - `cargo check -p katana-document-viewer --locked` passed.
    - `cargo clippy -p katana-document-viewer --locked -- -D warnings` passed.
    - `cargo test -p katana-document-viewer --locked -- --test-threads=1` passed.
    - `env RTK="rtk proxy" just storybook-release-acceptance-artifacts` passed and generated acceptance artifacts under `target/acceptance/`.
    - Verification logs are retained under `tmp/v0-22-33-html-file-preview-evidence/kdv-isolated-*.log`.
    - `CARGO_INCREMENTAL=0 cargo package -p katana-document-viewer --locked --allow-dirty` passed after clearing generated build artifacts from the isolated worktree.
    - `CARGO_INCREMENTAL=0 cargo publish -p katana-document-viewer --locked --dry-run --allow-dirty` passed and aborted upload due to dry-run.
    - Public state check: crates.io latest is still `katana-document-viewer 0.2.7`; GitHub Release `v0.2.8` is not found; remote tag `v0.2.8` is not present.
    - 2026-07-06 public state recheck: crates.io latest remains `katana-document-viewer 0.2.7`; GitHub Release `v0.2.8` is not found; remote tag `v0.2.8` is not present.
    - 2026-07-06 non-public recheck: KDV isolated worktree remains staged-only with no unstaged diff; `git diff --cached --check`, `just VERSION=0.2.8 release-target-check`, and `cargo test -p katana-document-viewer --locked release_publish_depends_on_release_check -- --test-threads=1` passed. `bash scripts/release/assert-crates-not-published.sh 0.2.8` confirms `katana-document-viewer 0.2.8 is unpublished`.
    - 2026-07-06 full gate re-run after the static CSS regression test addition:
      - `env RTK="rtk proxy" CARGO_INCREMENTAL=0 just storybook-release-acceptance-artifacts` passed and regenerated `kdv-isolated-storybook-acceptance-20260706.log`.
      - `env RTK="rtk proxy" CARGO_INCREMENTAL=0 KDV_RELEASE_DOD_SKIP_ACCEPTANCE_FRESHNESS=1 just VERSION=0.2.8 release-check` passed and wrote `kdv-isolated-full-release-gate-20260706.log`.
      - The full gate includes `direct_html_source_applies_static_css_without_metadata_body_text`, `direct_html_source_applies_id_and_inline_override_css`, `style_text_does_not_count_as_visible_text`, and `direct_html_requirement_features_reach_kuc_storybook_scene`.
      - Coverage summary passed with total line coverage `96.67%` against `--fail-under-lines 95`.
      - `cargo package -p katana-document-viewer --locked --allow-dirty` passed.
      - `cargo publish -p katana-document-viewer --dry-run --locked --allow-dirty` passed and stopped with `warning: aborting upload due to dry run`.
      - `bash scripts/release/assert-crates-not-published.sh 0.2.8` still confirms `katana-document-viewer 0.2.8 is unpublished`.
    - 2026-07-06 V8 boundary correction: KDV core no longer declares a direct `v8` dependency. `viewer_manifest_keeps_ui_vendor_and_runtime_dependencies_out` fixes this boundary by rejecting direct `v8`, `katana-ui-core`, `egui`, `winit`, and `vello` dependencies in `katana-document-viewer`.
    - 2026-07-06 V8 boundary verification: `cargo tree -p katana-document-viewer --locked -i v8` shows `v8 v150.0.0 -> katana-render-runtime v0.3.8 -> katana-document-viewer v0.2.8`; evidence is retained in `kdv-isolated-v8-boundary-20260706.log`.
    - 2026-07-06 final full gate after V8 boundary correction:
      - `env RTK="rtk proxy" CARGO_INCREMENTAL=0 just storybook-release-acceptance-artifacts` passed and regenerated `kdv-isolated-storybook-acceptance-20260706-v8-boundary.log`.
      - `env RTK="rtk proxy" CARGO_INCREMENTAL=0 KDV_RELEASE_DOD_SKIP_ACCEPTANCE_FRESHNESS=1 just VERSION=0.2.8 release-check` passed and wrote `kdv-isolated-full-release-gate-20260706-v8-boundary.log`.
      - The final gate includes `viewer_manifest_keeps_ui_vendor_and_runtime_dependencies_out`, `direct_html_source_applies_static_css_without_metadata_body_text`, `direct_html_source_applies_id_and_inline_override_css`, and `direct_html_requirement_features_reach_kuc_storybook_scene`.
      - Coverage summary passed with total line coverage `96.67%` against `--fail-under-lines 95`.
      - `cargo package -p katana-document-viewer --locked --allow-dirty` passed.
      - `cargo publish -p katana-document-viewer --dry-run --locked --allow-dirty` passed and stopped with `warning: aborting upload due to dry run`.
      - `bash scripts/release/assert-crates-not-published.sh 0.2.8` still confirms `katana-document-viewer 0.2.8 is unpublished`.
    - KDV `0.2.8` was locally ready only for the superseded static-preview scope. It MUST NOT be published or consumed for the browser-equivalent v0.22.33 viewer.
    - 2026-07-06 KDV publication route confirmation:
      - KDV `.codex/skills/impl-release/SKILL.md`, `.codex/workflows/impl-release.md`, and `docs/release.md` all state that release work must use a `release/vX.Y.Z` PR into `master`; manual tag creation and direct `cargo publish` are not the normal release path.
      - KDV `.github/workflows/release.yml` creates the tag, GitHub Release, and crates.io publication after a merged `release/v...` PR.
      - `scripts/release/publish-crates.sh` requires a clean worktree and `CARGO_REGISTRY_TOKEN`; this is invoked by the Release workflow, not by the Katana recovery worktree.
      - Current lightweight recheck passed: `just VERSION=0.2.8 release-target-check`, `bash scripts/release/assert-crates-not-published.sh 0.2.8`, `git diff --cached --check`, and `cargo metadata --format-version 1 --locked --no-deps` showing KDV `0.2.8` depends on `katana-render-runtime ^0.3.8` from crates.io registry.
      - 2026-07-06 continuation recheck: no KDV PR exists for `release/v0.2.8` or `feature/html-css-preview-v0.22.33-clean`; the isolated KDV worktree has 0 unstaged files and 26 staged files; KDV `0.2.8` still depends only on `katana-render-runtime ^0.3.8` from crates.io among the relevant renderer/runtime dependencies.
      - 2026-07-06 PR-preflight-local recheck: `git diff --cached --check`, `just VERSION=0.2.8 release-target-check`, `cargo test -p katana-document-viewer --locked direct_html -- --test-threads=1`, `cargo test -p katana-document-viewer --locked dependency_tests -- --test-threads=1`, `cargo tree -p katana-document-viewer --locked -i v8`, and `bash scripts/release/assert-crates-not-published.sh 0.2.8` passed. `direct_html` covered 21 tests including static CSS application and `<head>` / `<style>` / `<script>` body-text suppression; dependency tests covered KDV manifest/public API boundaries.
      - 2026-07-06 full release-check rerun: `env RTK="rtk proxy" CARGO_INCREMENTAL=0 KDV_RELEASE_DOD_SKIP_ACCEPTANCE_FRESHNESS=1 just VERSION=0.2.8 release-check` passed. The run covered release target / DoD assertions, fmt, clippy, AST lint, Storybook entrypoint, KUC adapter boundary, workspace tests, kdv-storybook tests, subagent harness checks, coverage, package, publish dry-run, and unpublished-crate assertion. Coverage total line coverage remained `96.67%`; `cargo publish --dry-run` stopped with `warning: aborting upload due to dry run`; `katana-document-viewer 0.2.8 is unpublished`.
      - 2026-07-06 logged full release-check rerun: same release-check passed again with raw log retained at `tmp/v0-22-33-html-file-preview-evidence/kdv-isolated-full-release-gate-20260706-continuation.log` (3714 lines). Key log evidence: DoD check line 8, static CSS / metadata suppression tests lines 931 and 2999, `checks passed` line 2130, coverage `96.67%` line 3692, package line 3696, publish dry-run line 3703, dry-run abort line 3712, unpublished assertion line 3714.
      - Superseded route: the KDV `0.2.8` release path is withdrawn for this goal. The active route is published KRR `0.4.0`, then KDV `0.3.0` browser adapter, then KatanA `v0.22.33` integration and evidence.

## 6. Final Verification & Release Work

- [x] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
  - Result: PASS. Split newly grown files back under 200 lines, fixed AST lint nesting/magic-number findings, and kept external KDV/KRR changes out of scope.
  - Verification:
    - `cargo test -p katana-core html_documents -- --nocapture`
    - `cargo test -p katana-core test_html_extension_detection -- --nocapture`
    - `cargo test -p katana-core render_ -- --nocapture`
    - `cargo test -p katana-ui --lib html -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_serial html_document_buffer_uses_kdv_preview_surface -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_parallel --no-run`
    - `cargo clippy -p katana-core -- -D warnings`
    - `cargo clippy -p katana-core --lib --tests --all-targets -- -D warnings`
    - `cargo clippy -p katana-ui -- -D warnings`
    - `just ast-lint`
    - `just fmt-check`
    - `git diff --check -- <touched paths>`
    - `./scripts/openspec validate release-tracking-v0-22-33-html-file-preview --strict --no-interactive`
- [x] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
  - Result: `KML_SCOPE=openspec/changes/release-tracking-v0-22-33-html-file-preview just kml-check` passed. No CHANGELOG update exists yet in the implementation PR phase.
- [x] 6.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
  - Result: `git push -u origin v0-22-33-html-file-preview` passed the normal pre-push hook.
- [x] 6.4 Create PR from Base Feature Branch targeting `master`
  - PR: [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317)
- [x] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
  - Result: PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317) CI passed after coverage補完テスト追加.
- [x] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
  - Merge commit: `30c01e73441777ed53a59a1d88e2ca1de16a30f2`
- [x] 6.7 Create `release/v0.22.33` branch from master
- [x] 6.8 Run `just VERSION=0.22.33 release` and update CHANGELOG (`changelog-writing` skill)
  - Result: version bump commit created; CHANGELOG EN/JA updated for 0.22.33.
  - Release preflight requires version-prefixed active OpenSpec changes to have no pending tasks. The change directory was renamed to `release-tracking-v0-22-33-html-file-preview` so the remaining release delivery tasks stay tracked without blocking the release branch.
- [/] 6.9 Create PR from `release/v0.22.33` targeting `master` — Ensure `Release Readiness` CI passes
  - PR: [#320](https://github.com/HiroyukiFuruno/KatanA/pull/320)
  - Current status: open; the 2026-07-21 headless release evidence is approved and the local update is ready for final review and push.
  - 2026-07-06 live status: PR #320 remains open with `mergeStateStatus=BLOCKED`. Release Readiness is green, but CI is not green (`Test and Build (ubuntu-latest)` failed; macOS/Windows builds cancelled).
  - 2026-07-06 CI rerun: failed workflow run `28696531525` was rerun. `Test and Build (ubuntu-latest)`, `Test and Build (windows-latest)`, and `Test and Build (macos-latest)` completed with `success`; PR #320 is now open with `mergeStateStatus=CLEAN`.
- [ ] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
  - Ready after the approved local update is pushed and all PR checks pass.
- [ ] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
  - Ready after PR #320 merges; release only `v0.22.33`.
- [x] 6.12 Withdraw erroneous GitHub Release/tag `v0.29.0` before treating `v0.22.33` as the next release.
  - Result: done. `gh release delete v0.29.0 --cleanup-tag --yes` completed successfully.
  - Verification:
    - `gh release view v0.29.0 --json tagName,url,isDraft,isPrerelease,publishedAt` -> `release not found`
    - `git ls-remote --tags origin v0.29.0 v0.22.33` -> no matching remote tags
- [/] 6.13 Re-run browser-equivalent user-intent verification and Markdown-only tool isolation before release approval.
  - Required minimum: published dependency/runtime guard、source-boundary tests、KDV adapter/KatanA integration tests、OpenSpec strict validation、deterministic headless interactive evidence、release readiness check。
  - Current result: published KDV `0.3.1` and KRR `0.4.3` passed the registry-only 60/60 headless acceptance. CSS, accordion, V8 DOM mutation, committed Japanese IME input, `preventDefault()`, scrolling, complete same-document and external fragment origins, raw KRR frame pixels, composed screenshot pixels, fragment state preservation, reload, and exact resize were independently verified and approved by the user.
  - Browser acceptance must cover CSS layout, accordion, JavaScript DOM mutation, text/IME input, link navigation, `preventDefault()`, reload, exact viewport resize, complete action frames, and no overlapping/clipped text.
  - Historical static-path verification completed (insufficient for the corrected browser contract):
    - `cargo fmt --all -- --check`
    - `cargo check -p katana-ui`
    - `cargo clippy -p katana-ui -- -D warnings`
    - `cargo test -p katana-ui --test ui_integration_serial -- --nocapture`
    - `cargo test -p katana-ui --lib html -- --nocapture`
    - `cargo test -p katana-ui refresh_preview_routes_html_file_to_kdv_preview_surface -- --nocapture`
    - `cargo test -p katana-ui full_refresh_preview_routes_htm_file_to_kdv_preview_surface -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_serial kdv_preview_engine_treats_html_path_as_html_document -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_serial html_document_buffer_uses_kdv_preview_surface -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_serial html_preview_surface_is_not_reused_as_markdown_diagram -- --nocapture`
    - `scripts/screenshot/run.sh --request scripts/screenshot/examples/v0-22-33-html-file-preview.json --output tmp/v0-22-33-html-file-preview-evidence`
    - `cargo tree -p katana-document-viewer --edges normal,build,dev | rg "egui|eframe|winit|vello"` -> no matches
    - `cargo tree -p katana-render-runtime --edges normal,build,dev | rg "egui|eframe|winit|vello"` -> no matches
    - `./scripts/openspec validate html-file-preview --strict --no-interactive`
    - `./scripts/openspec validate workspace-file-filter --strict --no-interactive`
    - `./scripts/release/preflight.sh 0.22.33`
    - `GITHUB_ACTIONS=true ./scripts/release/check-pr-ready.sh 0.22.33`
    - `kml check --config .markdownlint.json openspec/changes/archive/2026-07-04-release-tracking-v0-22-33-html-file-preview openspec/specs/html-file-preview/spec.md openspec/specs/workspace-file-filter/spec.md`
    - `git diff --check`
    - `cargo test -p katana-ui --test ui_integration_parallel -- --nocapture` -> `137 passed; 0 failed; 2 ignored`
    - Clean detached worktree at `origin/release/v0.22.33`: `cargo test -p katana-ui --test ui_integration_parallel -- --nocapture` -> `137 passed; 0 failed; 2 ignored`
    - GitHub Actions workflow run `28696531525` rerun -> completed `success`
    - 2026-07-06 current-state rerun: `env CARGO_TARGET_DIR=tmp/cargo-target-v0-22-33-html-evidence cargo test -p katana-ui html -- --nocapture` -> passed. Covered 28 matching unit tests, 3 matching parallel integration tests, and 10 matching serial integration tests, including `.html` / `.htm` visibility/openability, persisted-settings fallback, KDV preview routing, and `html_document_buffer_uses_kdv_preview_surface`. The isolated target dir was used because the default `target/debug` still contained stale build output pointing at a removed recovery worktree `Info.plist`.
  - Current gate status: `scripts/release/check-html-browser-release-contract.sh 0.22.33` passes with KDV `0.3.1` / KRR `0.4.3` in both application and headless harness locks and rejects stale patch lines.
- [/] 6.14 Update KatanA only after the KRR/KDV browser chain is publicly available.
  - Required minimum: `Cargo.toml`/`Cargo.lock` resolve crates.io KDV `0.3.x` and KRR `0.4.x`; no path/git override exists; the runtime stays in-process Rust/V8 with no Chromium, WebView, helper binary, or browser archive.
  - Current status: KRR `0.4.3` and KDV `0.3.1` are published and verified. KatanA and its headless harness resolve both from crates.io with checksums and no path/git override; the registry-only rerun passed 60/60.
  - Release target: only adjacent `v0.22.32 -> v0.22.33` is valid. Withdrawn `v0.29.0` must remain absent and is never a fallback target.
- [/] 6.15 Re-run final self-review and all KatanA release gates after browser integration.
  - Current status: fmt, strict clippy, AST lint, fixture integration, unchanged strict 100% coverage, OpenSpec 54/54, SemVer contract tests, Linux workspace tests, and the registry-only 60-step headless contract pass with KDV `0.3.1` / KRR `0.4.3`. User screenshot approval was received on 2026-07-21; final diff review and post-commit PR checks remain.
  - Required final evidence: shell contract tests、OpenSpec strict validation、KML、AST lint、fmt、strict clippy、relevant unit/integration tests、deterministic headless capture、`git diff --check`。
