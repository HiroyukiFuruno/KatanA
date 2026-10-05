# Existing HTML navigation target ownership

## 最新checkpoint（2026-10-06 JST、公開HEAD c2ed191b）

cfg(test)共有runtime guardの1行修正27a0b598と証跡c2ed191bは通常push58363 exit0で公開HEAD一致、ahead/behind0/0、stash0/master cleanを確認。製品コード、既存2秒wait、pixel/dirty/fragment assertは変更していない。変更後公式coverage/supply-chain86558 exit0、UI1059/既存ignore2・実export13・parallel143/既存ignore2・serial18、meaningful未実行0/strict文書100%/未実行0、供給網4分類PASS。rawはtmp/html-preview-runtime-guard-current-coverage-2026-10-06.log。native JSON15508304bytes SHA d67853e9ecfd20eaa51f6177684868d6ab2baa547b968c4e810247407fa352b7。html_navigation全fileは478/507lines・755/800regions・44/44functionsであり、全file100%とはしない。

現HEAD review PRR_kwDORm09y88AAAABQvAyZAはmanual-target公開方針の重複P2を検出。thread PRRT_kwDORm09y86pIe7Bへreply4186624366後、全thread/commentsを再取得し次page無し、元方針threadと合わせ未解決2件/P0P1未解決0を確認。ユーザー判断待ちの同一公開方針であり、未修正・未resolveを維持する。PR346はDraft。新CI37343717559はUbuntu buildと全3OS lint・供給網・readiness成功、macOS/Windows buildとCodeQL進行中。旧macOS失敗の解消はcloud結果未確認のため先取りしない。

KDV v0.5.12のRelease37332907624 success、GitHub Release16:58:52Z、registry非yank/checksum4d39820039dee936b5ae1b7d9656751223421b5140c2b25f91d15ebe7519d635をライブ確認し、固定依存と両lockを更新した。差分は版/checksumのみでlocked metadataは両方exit0。製品Office解放変更なしを公式tag比較で確認。新graph全gate・原本再受入・正式commit/push/current reviewは未完了。以下のKDV0.5.11原本RSS FAIL2を、この新公開版や検査修正で解消扱いしない。親同PIDのcold/各正常close直後を分類する任意診断observerはscreenshot検査ハーネス限定で実装中、通常受入の元基準を維持する。

## 最新検証（2026-10-06 JST、HEAD 5f9ccff5）

source62ab1a14と証跡5f9ccff5は署名Gで正式統合し、通常push6373はexit0、GitHub HEAD一致・ahead/behind0/0を確認した。P2 thread PRRT_kwDORm09y86pGLOgへreply4186114798を入れてresolveし、全件再取得で解決を確認。現HEADのCodex本文レビュー5998463527（2026-10-05T16:17:26Z、Reviewed commit5f9ccff5f8）はmajor issuesなし。manual-target公開方針P2はユーザー判断待ちのまま残す。

変更後の公式test-integration/coverage/supply-chain55122はexit0。実worker fixture8、native UI1059/既存ignore2、実export13、parallel143/既存ignore2、serial18が成功し、meaningful未実行0・strict document surface100%/未実行0、供給網4分類PASSを確認した。既存duplicate警告を隠さず保持。rawはtmp/html-navigation-collision-preview-release-gates-2026-10-06.log。native coverage JSON15508045bytes、SHA2d867dd518e3b26cc21397a7779272abf2a518388a577ff9db878da7ea993041を保存。html_navigation全fileは477/506lines・753/798regions・44/44functionsであり、全file100%とはしない。

公式package-mac85190とfresh arm64 startup3405はexit0。main SHA f9d09e2eddbdd44a76d6998d38fcc8397ff1522a967c10bae21f0d46e238915a、worker7191cbaf、両universal/minos13.0/deep strict ad-hoc署名を確認した。startup peak195184KiB/font27093388bytes/Office worker0、owned PID43543終了。空workspace起動を文書受入・全5native・clean-machine・95点へ代用しない。Terms承認は代行していない。

原本HTML76489はexit0、初回1.337秒・fragment1.641018334秒・正常close0.830844750秒、全9操作・idle資源0・残存process0を確認した。原入力SHA c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012、viewport/fragment/deadlinesは不変。これはin_process_hostであり、packaged-main文書受入や独立95点ではない。

### 新規失敗：原本Office5の終了後RSS

公式scripts/screenshot/run.sh、原request SHA f58b929fab7430afc4eb5893f49bccd0d7a7efd6a8cf2fddc29339c7a3641d0c、新runner SHA24ac73e5f99d67178f5fe1120fa79a35a0fa5313fa2501990ce7f7ffd4a7982b、実worker830426adで2回測定し、どちらもactual exit1。重い検査の同時実行やDEBUGはない。固定registry KDV0.5.11/KRR0.4.23/KUC0.4.1を維持した。

- 初回：baseline104880→closed304560KiB、delta199680KiB。
- 反復：baseline104656→closed304624KiB、delta199968KiB。

元上限196608KiBを両方で超えた。5文書frameと各close操作を観測し、最後のresource/surface/worker/frame/texture/cache counterは0だが、step20のRSS assertion失敗でstep21 quitには到達していない。資源counter0だけで受入成功や原因特定としない。rawはtmp/html-navigation-collision-preview-office-five-2026-10-06.logとoffice-five-repeat-2026-10-06.log。既存KDV #59担当へ原入力・binary identity・2回の生ログを送付し、既存担当で調査を継続中。KDV0.5.12はRelease verify中で未公開のため未採用、公開だけで#58/#59完了とも扱わない。

### 新規失敗：現HEAD macOS CI

CI run37338244362/job111858559878はRun testsでexit101。1058pass/1fail/既存ignore2、file_navigation_to_an_open_dirty_target_preserves_target_stateが「timed out waiting for the HTML browser frame」。rawはtmp/html-preview-current-macos-ci-failure-2026-10-06.log。local focused8件/full coverage成功をこのcloud失敗の成功証拠へ代用しない。既存限定担当へ同file内の原因調査・最小修正を依頼し、timeout延長・固定待機・Mock・assert削減・製品コードのテスト都合変更は認めない。PRはDraftを維持する。

限定担当とmainの差分/caller確認で、新しい実描画テストだけ既存RenderEnvLockを取得していないことを確認した。他のHTML描画回帰と同じhtml_browser_runtime_test_guard()を1行追加し、製品コード・2秒wait・実RGB/dirty/origin/fragment/所有assertを維持した。focused1件/関連8件GREEN、main公式実worker/workspace入口22841も8pass/1836filtered/39suites、actual exit0。fmt/diff check成功。rawはtmp/html-preview-current-macos-ci-fix-suite-green-2026-10-06.logとtmp/html-preview-runtime-guard-canonical-2026-10-06.log。共有lock不足は確認済みだが、macOS全CIの解消は変更後CIが成功するまで未確認として残す。通常source commitと次のnormal pushで正式ゲートを通す。

テスト修正は通常commit91873 exit0・署名Gの27a0b59847bf41471390d3b3f0125648ab8c256dへ正式統合。証跡は別commitとし、normal push/新HEAD cloud結果は別checkpointで確認する。OfficeRSSの2回失敗と独立受入/公開DoDは未完了のまま維持。

## Finding and scope

Current-HEAD review 5416352520 on c8335e48 reported that navigating from local HTML A to already-open B preferred A's document record, renamed it to B and removed B's preview. This produced two records with the same path. The review-specific GraphQL query returned no inline comments, and the complete paginated fetch returned no thread for this finding. Its review URL is https://github.com/HiroyukiFuruno/KatanA/pull/346#pullrequestreview-5416352520.

Only `crates/katana-ui/src/app/action/html_navigation.rs` changes. Existing target lookup now precedes source fallback. For a distinct source/target collision, the existing target is activated without replacing its document record, dirty buffer, pinned flag or per-tab layout. Its existing browser session receives the requested source/fragment through the existing preview operation. Source records and previews remain owned by their original paths. Missing-target navigation and same-path reload retain the existing replacement path. No public API, sibling dependency, rendering contract, acceptance limit, lint or coverage exclusion changes.

## Regression and integration review

The bounded child observed the original duplicate-path regression fail with two target paths, then a target-first intermediate implementation passed its focused navigation suite. The main integration review rejected that intermediate implementation because it could overwrite a dirty target's unsaved content. The child added a real temporary-file/file-URL regression and observed the intermediate implementation replace `unsaved-target` with the disk content. This second RED is explicitly against the intermediate target-first implementation, not the original HEAD; raw receipt is `tmp/html-navigation-collision-dirty-intermediate-red-2026-10-05.log`.

The repaired collision branch preserves the dirty target and both document/preview owners, while forwarding B#details and retaining target history, pinned/split/pane order. A separate same-path reload regression requires one record, replacement content and old-to-new history. The child filtered suite passed seven tests with zero failures/ignored tests. Its final raw output is retained at `tmp/html-navigation-collision-child-green-2026-10-05.log`. The main inspected the actual diff and RED/GREEN logs, not only the child's completion message. Formatting and diff checks passed.

The main canonical `with-office-test-worker.sh` followed by `just T=html_navigation test-specific` exited 0: seven passed, 1836 filtered across 39 suites. JOBS=2, CARGO_INCREMENTAL=0, existing debug/strip profiles and `-D warnings` were retained. Raw receipt: `tmp/html-navigation-collision-canonical-green-2026-10-05.log`. Main self-review found the final branch scoped to distinct existing-target collisions; source/path ownership and per-tab state remain unchanged, and no mock, fixed wait, disabled check or threshold relaxation was added.

Normal source commit completed with exit 0 and signature G as `834b58b8`, passing the existing commit verification hooks without bypass. Raw receipt: `tmp/html-navigation-collision-source-commit-2026-10-05.log`. The preceding packaged/original-HTML evidence was separately committed as signed `2d3e53e8`; it is not evidence for the new navigation source.

Normal push, individual review response, fresh review retrieval and changed-source coverage/packaging/acceptance remain subsequent checkpoints. None are inferred from focused success. Earlier c8335e48 packaged startup/original-HTML proof remains historical for this additional Rust change. PR346 stays Draft, and manual-target policy, full native distribution acceptance and independent scores remain unresolved.
# Current-head follow-up at 490df76a

## Preserved-buffer preview correction

The first normal commit was rejected (exit1) by the unchanged AST200-line responsibility limit: production html_navigation.rs became210 lines. It is not treated as a commit or gate success. The duplicated browser-source creation is being consolidated within the same responsibility; no exclusion, limit change, hook bypass or arbitrary file-line relocation is used. Raw failed commit and RTK AST diagnostics are retained.

The consolidation removes duplicate source construction from file navigation: file loading delegates to the common replacement path, which selects the preserved target buffer or incoming source, validates one browser source before any document mutation, then performs the existing collision/normal branches. Production responsibility is184 lines excluding tests. The unchanged seven-test suite passes again after this refactoring; an invalid-origin preservation regression and normal AST recheck are next. Earlier first-draft canonical success is not substituted for the refactored source's verification.

The added invalid-origin regression verifies the complete existing document, active path and preview history remain unchanged. The final child suite passes8 tests and the unchanged AST linter passes23 tests; formatting/diff validation passes. Main inspected both raw outputs and the final common-constructor diff. Raw logs are tmp/html-navigation-collision-invalid-origin-green-2026-10-06.log and tmp/html-navigation-collision-ast-lint-2026-10-06.log. The original-input/process contracts independently pass24+7 tests. Final canonical execution and normal commit/push remain pending at this checkpoint.

The final refactored-source canonical real-worker/workspace test completes with actual exit0:8 passed/1836 filtered/39 suites, preserving JOBS2/CARGO_INCREMENTAL0/-Dwarnings/debug0/strip-none settings. Raw is tmp/html-navigation-collision-preview-final-canonical-2026-10-06.log. Main self-review passes after checking common-constructor error-first ordering, requested origin/fragment and unchanged normal/collision ownership behavior. Normal source commit retry is in progress; no prior coverage/package/current-review result is attributed to this new source.

Normal source commit retry succeeds with actual exit0 and signatureG at62ab1a14fd9f9b7a20eb7244aa6e368f7c4e145c, without bypassing formatting, Clippy or AST hooks. Raw receipt is tmp/html-navigation-collision-preview-source-commit-retry-2026-10-06.log. Source is formally integrated; evidence is committed separately. Push, individual thread reply/resolve and fresh review, changed-source full coverage/package/original-input and remaining release acceptance are not yet claimed.

Main canonical `with-office-test-worker.sh "just T=html_navigation test-specific"` passes7 tests/1836filtered across39 suites, actual exit0. The current diff and callers were reviewed: no public API change, same-path/missing-target behavior retained, constructor errors visible, no criteria/exclusion relaxation, no Mock/snapshot comparison and no new production test port. Format/diff checks pass. Source and evidence remain separate concerns for normal signed commits; push/current-review/full changed-source evidence remain pending.

The existing target now creates its browser source from the preserved buffer and the requested destination origin, including its fragment. Source/target document state and preview ownership remain intact; missing-target and same-path branches are unchanged. Constructor failures remain explicit status errors, not silent fallback.

An actual temporary file with blue saved contents and red unsaved target contents reproduces the current490df76a defect: the rendered RGB response assertion fails (0passed/1failed). Restoring the correction passes all7 html_navigation tests, including the actual rendered red pixels and exact fragment-bearing origin, original buffer/dirty/pinned/layout checks and same-path history. This is a response-based real browser test, not a snapshot comparison, mock or relaxed deadline. Raw RED and GREEN are retained as tmp/html-navigation-collision-preview-red-2026-10-06.log and tmp/html-navigation-collision-preview-suite-green-2026-10-06.log. Formatting and diff validation pass. Canonical real-worker workspace verification and formal integration/current-review/full changed-source gates remain in progress; preceding490df76a package/full-gate results above are not transferred to the corrected source.

The normal push succeeded and GitHub HEAD matched490df76a, upstream0/0, master clean and stash0. Body-only review5416352520 had no inline comment/thread; its individual linked response is issuecomment5997377591. A new review was requested with issuecomment5997388246.

Official `JOBS=2 CARGO_INCREMENTAL=0 just check-full` completed exit0: native UI1058 (existing2 ignored), actual export13, parallel143 (existing2 ignored), serial18, real-worker fixture8, meaningful uncovered0 and strict document-surface100%, Linux actual-worker workspace tests, Windows test-inclusive crosscheck and supply-chain checks passed without criteria/exclusion changes. Native raw JSON15504003bytes SHA256b90eb4c2533f816b1b63818b8aefe281141a2575dfd3259c194318508766087c is retained. The whole html_navigation.rs raw coverage is444/476 lines,694/745 regions and41/43 functions; it is not claimed as100%.

Official macOS package and strict deep ad-hoc verification passed. Universal main SHA256964eae7d9c83384e4b3dd75c7de75ad5adaeb28bc873ae7e8bd48f6257231fe3 and worker7191cbafe1f79f3cda93f90961bea13f6d81318904a3f8fda5dc4eeff8c320ce both retain minimum macOS13.0. Fresh arm64 empty-workspace smoke exited0 with peak207200KiB, font27093388bytes, Office workers0 and owned PID termination verified. This is not packaged document/clean-machine/all-target acceptance.

Fresh original-HTML in-process acceptance exited0: first1.296s, fragment1.660793541s, normal close0.830083s, all9 operations and idle resources/process-group closure passed. Runner SHA256a2e037038f62bffc2a8deed7a9f317ef4bc4defa4bad8c2b2b3eb17cedbae1ed and worker830426ad5c43e705ec69f7cc67d75952e0578b175357c1403aa5a04133d52955 were rebuilt. Input, viewport, fragment and deadlines are unchanged. Raw evidence is under tmp/html-navigation-collision-original-html-2026-10-05. This is explicitly in_process_host, not packaged-main acceptance or independent95-point scoring.

Fresh current-head review PRR_kwDORm09y88AAAABQt3wcw reports P2 PRRT_kwDORm09y86pGLOg: preserved dirty target buffer and disk-derived preview source disagree. Buffer-derived navigation-source correction with destination URL/fragment preservation and actual-file regression is in progress. Manual-target publication-policy P2 remains a separate human choice. This follow-up does not claim either finding resolved or any subsequent changed-source gate passed.
