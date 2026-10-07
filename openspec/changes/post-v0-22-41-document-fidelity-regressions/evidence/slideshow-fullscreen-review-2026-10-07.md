# スライドショー状態遷移と診断の資源隔離

## 今回のcurrent review

PR346、公開181391e5へのreview5430016913。

- PRRT_kwDORm09y86pg4Qo / 4196609706: sidebarが非対応文書への遷移でflagだけ消し、OS fullscreenを戻さない。
- PRRT_kwDORm09y86pg4Qv / 4196609713: 無文書でSlideshowが有効で、空workspaceからFullscreen(true)へ入る。
- 別のmanual-target方針二件は未決を維持し、この修正の採否と混同しない。

## 最小修正と回帰

通常署名付きcommit e2d3d361fef28938e4edbc07ba3dc0f30e44d5e4。

- sidebarと直接action guardは、Slideshowが実行中かつ開始前が非fullscreenの場合だけFullscreen(false)を送って終了する。
- 無文書ではSlideshowだけ不可、他4メニューの既存None契約は維持する。
- 新しいprivate cfg(test) fixtureは188行。既存の未接続slideshow testの接続/API修正案は取り込まず元の状態へ戻した。
- 同じfixture/旧製品分岐で2PASS/5assertFAIL、actual101。raw `/tmp/slideshow-review-red-20261007.log`。None可用性、無文書Fullscreen(true)、直接action復元、反復復元、sidebar復元の固有assert。
- 修正後mainの正式worker付き入口session72143 actual0、7PASS/1130filtered/0.06秒。raw `tmp/slideshow-current-root-green.log`。
- 文書無し、HTML/DOCX/XLSX/PNG/PDF、開始前fullscreen維持、初回だけ復元、反復sidebar/action、Markdown/PPTX可用性を実egui viewport commandで確認。
- これはunitモデル上のcommand出力検証であり、本人native appや全OS packaged受入の完了ではない。

## 元macOS CI失敗を保持

181391e5 macjob112323232248は元2秒でfirst frame2041ms timeout、1127PASS/1FAIL/ignore2。raw `tmp/181391e5-macos-job-112323232248.log`。

元failure PID22676 sampleは収集終了後のsymbol処理で5秒上限に達し、0byte出力だった。後続別process159.312ms成功を原失敗へ代用しない。`tmp/181391e5-current-ci-audit/analysis.md`にwindowと限界を記録。

現runの実child profilerはstartup窓と約0.65秒重複したが、旧4a/683は非重複でも失敗した。この交絡は反復失敗の必要原因ではなく、製品修復や原因確定とはしない。

通常署名付きcommit 1c6ab10e8aa3a7fdbc3a6841d36a443175eeb2deで、実child sample testだけ既存RenderEnvLockを取得する。helper本体/製品コード/原2秒/RGB/assert/ignore/2threadsは不変。

- 正式worker付きhtml_navigation::全group session38894 actual0、11PASS/1126filtered/2.13秒。raw `tmp/181391e5-profiler-isolation-html-group.log`。
- 公式AST23 actual0、`tmp/slideshow-profiler-current-ast.log`。
- selected UI all-targets strict Clippy/fmt session33731 actual0、`tmp/slideshow-profiler-current-clippy-corrected.log`、`tmp/slideshow-profiler-current-fmt.log`。
- 先行Clippyは既存justのstrip=none設定を指定し忘れproc-macro E0463/actual101。入口の設定を訂正し、ソース/ゲート緩和やregistry改変は行っていない。失敗raw `tmp/slideshow-profiler-current-clippy.log`を保持。

新HEAD通常push/current review各reply-resolve-fresh/全CI・coverage・package・本人受入・公開後処理は別の未完工程。旧181 packageや成功を後続graphへ流用しない。
