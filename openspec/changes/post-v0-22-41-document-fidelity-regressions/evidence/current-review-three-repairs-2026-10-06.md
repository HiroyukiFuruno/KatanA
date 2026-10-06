# c07 current review追加3件の修正検証

## 最新統合と追加レビュー

画像監視候補は2a38304b、公開後host契約は53f754f8へ通常hook・署名Gで関心別commitした。最終watch-final-type AST23PASS/strict impacted Clippy PASS/fmt PASS、各actualexit0。60880のtype_complexity 1FAILはprivate型別名で補正し、最終自己reviewをwatch-overflow-self-review-2026-10-06.mdに記録した。機能検査はroot1445の27PASS。通常push、新HEAD cloud/coverage、個別replyresolve、実packagedと公開は未完。

画像watch overflowの最終root候補は、通知あふれと本当の監視失敗を別eventとして扱う。失われたRegisteredを含むpending登録は次pollで回復し、queue外へ保持した登録結果は後続Registered/Failedとの通知順を維持する。旧世代/resetは拒否し、既存16件/64MiBの画像結果cacheは不変。新mapはドロップした登録成功/失敗の通知用であり画像cacheではない。

root自己reviewで、既に適用した実失敗後のRegistered自体がdropすると旧errorが残る境界も追加検出した。96340は新assertion RED actualexit101、tmp/watch-registration-overflow-main-red.log / RTK tee1791269964。queue外結果を成功ackも含む一つのResult通知mapへ補正し、root1445は全loader27PASS/40suites/2.86秒 actualexit0。先行59802 strict lintはmap_entry/unnecessary_mut_passedの2件FAILで保持し、get_mutによる置換とimmutable引数へ規約準拠で修正。最終AST/lint/fmtは60880で回収中。新graphへの旧26PASS/旧AST成功の流用はしない。

root全loader96878は22PASS/1FAIL（idle GUI repaint5秒、tmp/watch-overflow-main-loader-failed-raw.log）。notify8.2.0の実ソースでFSEvents watch/unwatchが全streamをstop/runし未配送eventをpurgeすることを確認し、監視先追加時の既存targetと解除成功時の残存targetへOverflowを送る。解除成功分は一度にまとめ、解除失敗ごとの全画像再読込ループを追加しない。backendの明示Rescan flagも処理する。各回帰は実RecommendedWatcher/実PNG/loader pollを使用するが、topology/rescan producer契約と実GUI atomic通知を区別する。

新topology2回帰は通知補正前の旧挙動で0PASS/2assertion FAIL（82750 actualexit101、tmp/watch-topology-main-red-raw.log）。最初の補正後65326は23PASS/2FAILで、同じprocessの別directory監視変更によるrevision変更が「readだけは変更しない」というテスト前提に混入した。実backendを使う回帰は既存RenderEnvLockで共有OS監視環境を隔離し、二つのlive loader間のtopology変更は追加回帰内で検証する。条件・timeout・assert・case数は削減しない。最終root70647は全26PASS/40suites/2.55秒、85507は正式AST23PASS/4suites/12.56秒、いずれもactualexit0。raw tmp/watch-topology-main-loader-final.log / watch-topology-main-ast.log。agentの推定1791269307 logは存在しないため検証証跡へ採用しない。厳格lint/fmt・通常commit/push・thread reply/resolve・fresh review・新HEAD全gateは後続であり、実配布成功と主張しない。

公開後scopeの独立候補は `post-release-host` をopt-inとして追加した。旧strict/post-release-evidenceを保持し、延期を許すのは固定上流3.2と対応する実装marker完了済みの混合タスクだけ。証拠生成、公開後作業、未知タスクの未完は拒否し、既存全target/Office/原本hash/Chrome参照identity/入力/正常close/RSS条件を維持する。`check-pr-ready.sh --post-release-host` はstable GitHub Releaseの存在を要求し、既存readiness workflowを自動変更しない。

既存venvでgate30、flow9、divergence10、legacy acceptance48件はそれぞれactualexit0（tmp/post-release-host-{gate,flow,divergence,acceptance}-final.log）。shell syntax/diff checkもexit0。合成unit fixtureの回帰であり実packaged receipt・公開成果物・本人受入の証明ではない。候補は未commitで、v0.22.42は未公開のため実post-release-host成功とは扱わない。

最新ed8e35c1は通常push85912 actualexit0でlocal/upstream/PR346一致・ahead0/behind0。同一poll画像P2はreply4192050416→resolve→fresh全1page取得で解決確認した。AI review5424436693（chatgpt-codex-connector、ed8e35c1）は画像watch overflow P2 PRRT_kwDORm09y86pVutUを新たに指摘した。旧manual-target方針二件とこの新P2は未解決であり、Ready/公開は未完了。

ed8e35c1の公式coverage71777はactualexit0。UI lib1106PASS/既存ignore2、parallel143PASS/既存ignore2、serial18PASS、strict document surface100%/未実行0、既存meaningful gate PASS。raw tmp/ed8e35c1-current-coverage.logとtmp/ed8e35c1-coverage-report.jsonを保存した。生reportの全workspace line値と文書surface/meaningfulの判定は別物であり、全workspace生値100%とは主張しない。後続overflow修正graphや実配布/本人受入には流用しない。

新watch overflowは実PNGがReadyになった後、元productでwatch_errorが残るassertion REDを取得（tmp/watch-overflow-recovery-actual-red.log）。compile errorの初稿はRED証拠に数えない。二つの通知queue経路、lost Registered/pending、旧世代、genuine監視失敗保持の回帰と最小修正を継続する。

旧3件は通常push48627 exit0でHEAD04269741へ公開し、各返信4191620938/4191621452/4191622006とresolve後の全1page取得で解決済みを確認した。最新042レビューのP2同一poll結果欠落(PRRT_kwDORm09y86pUrOm / discussion_r4191642495)は別指摘として対応する。

同一pollに17個以上の小画像結果を受けた時、初consumer前に先行結果を退避してしまう問題を実texture経路で再現した。表示用active所有をResultへ一般化し、成功/失敗を初consumerまたはframe-graceまで保持する。新規map/allocator/公開APIは追加せず、reuse上限16件/64MiBを維持する。5追加回帰は同一poll全texture・次frame同TextureId・Failed非再queue・grace/reset/pathrevisionを検証する。実file decodeやnative GPU性能の証拠とは区別する。

最終公開依存graphで公式loader16PASS/40suites/2.13秒(session10252)、AST23PASS/17.12秒(16869)、strict impacted Clippy exit0(23335)、fmt-check91414/diffcheck exit0。先行watcher期待pixel失敗1件は原因未確定で、今回の成功だけで修復済みとは主張しない。関心事別署名付き通常commit67841c51、最新互換両lock3件aca70d84。通常push/新thread個別replyresolve/新HEAD全gateはまだ後続。

公開042のmacOS通常CIは1096PASS/1FAIL/既存ignore2、dirty-target初frame待ち2034ms/idlefalse/generationNoneで失敗。失敗後別process DEBUG成功0.31秒は元失敗stageの証拠ではない。Linux成功、Windows進行、元2秒/cold/判定不変で診断を続ける。

## 対象と未完了

PR346のc07b5028 current reviewで追加された小画像表示所有、Physics数式、Unicodeフォント名照合を対象とする。今回公開のKatanA修正であり、KDV/KRRの次期Issueを実装しない。通常commit/push、各thread reply/resolve、fresh review、新HEAD CI、packaged受入、公開は別工程で未完了。

小画像3b2a0341、MathJax177a0e26、Unicodeフォントcc8e9123へ通常hook・署名付きcommit済み。MathJaxの最初の未公開commit222c0c60には処理中の次関心事stageが混在したため、直前3b2a0341へsoft resetして作業内容を保持し、177a0e26/cc8e9123へ分離し直した。stash/new branch/強制push/hook迂回は用いていない。通常pushとcurrent review対応は未完了。

## 変更

- 小画像も再利用cacheとは別の表示中所有へ登録する。16件/64MiBの再利用上限を維持し、17枚の同TextureId、非表示後grace終了とreset解放を検査する。1pixelの所有回帰であり17枚大型GPU受入ではない。
- MathJaxの既存Physics importに対応するpackage登録を復元し、生成bundleも更新する。sourceと別関数のbundleを実際に呼び、qty/dvを検査する。実MathLogicOpsの小stack経路でもinline/block両方を検査する。
- フォント照合を共有FontFamilyIdentityのNFD→default case folding→NFDへ統一する。既存caseless/unicode-normalization registry版を直接依存へ追加し、両lockの依存edgeを同期する。合成/分解アクセント、sharp-s/SS、style区別、metadata/resolver/lease/painter/requestを検査し、testportや公開APIを追加しない。

## root実行結果

- MathJax source/generated bundle: bun run test、4PASS/22expect、exit0。実アプリ経路: just T=mathjax_backend test-specific、session10331、10PASS/40suites/0.78秒、exit0。初回inline dv期待幅の1FAILは表示別2.8ex/3.548exへ期待値を補正し、記録を消していない。
- 正式AST: just ast-lint、session29799、23PASS/4suites/12.39秒、exit0。初稿file length、public free helper、fixture数値違反は規約に合わせ補正し、除外/許容ルールを追加していない。
- just fmt-check session38937、just lint-impacted session73275、git diff --checkはexit0。Lintは全impacted4packagesのstrict Clippy。
- just test-ui session36972は1095PASS/2FAIL/既存ignore2。2FAILはOffice workerがdeps隣に存在しないためであり成功扱いしない。既存scripts/ci/with-office-test-worker.sh経由の全UI再検証session6855はexit0、1286PASS/既存ignore4/2196filtered/10suites/107.21秒。libの全件、image watcher、追加3件、実Office worker経路を含む。この成功をクラウドのcoldタイムアウト原因の解消やpackaged受入へ代用しない。
- 新JS回帰の既存Biome対象fmt/lint（session86276）、bun run typecheckはexit0。
- 小画像旧実装のassertion REDはtmp/small-slideshow-owner-red.log。先行loader全体87785は10PASS/既存watcher通知1FAIL、単独50248PASSだけで通知欠落原因の解消とはしない。
- Unicode初稿の3assertion REDはtmp/font-canonical-identity-red.log。先行8711/font155PASSは最後の構造補正前の結果なので最終graphへ代用しない。

## 既存CIの未解消

c07 macOS CI112090923848はHTML初回frame待ち2007ms/受理true/idlefalse/generationNoneで失敗した。KDV公開0.5.12のAdapter Dropはcloseとworker.joinを行うことを一次コードで確認し、非同期Drop残留という仮説は支持されなかった。原因監査は継続し、2秒期限・cold条件・assert・quality基準を変更しない。Linux成功/Windows進行を全3OS成功と扱わない。

## 通常push失敗後の回収

- 通常push23466はexit1。Linuxの小画像回帰が初表示前に任意のcache退避へ当たり、1079PASS/1FAIL/既存ignore2で停止した。remoteはc07のままであり公開・reply/resolveの成功ではない。
- 最初に所有回帰の組み立てを実build_textureへ変更した7f1b46ab。mac focused40544、Linux focused37017は各1PASS/exit0だったが、独立監査で初回consumer経路の検証が抜けると判明したため、この案は撤回した。成功を製品修復に代用しない。store_result→公開textureの元回帰を復元し、新規受信キーを即時退避から保護する最小製品修正を検証中。17件所有・同TextureId・非表示grace解放・16件/64MiB上限を変更しない。
- macOS CIが通常suiteで失敗した場合に限り、既存DEBUG描画traceをexact test/nocaptureで取得する別stepを追加。正式契約は旧workflowで1FAIL→修正版7PASS。通常suiteはDEBUG無し、continue-on-error無し、元cold/2秒/dirty保護assertを維持する。診断の成功で通常suite失敗を取り消さない。
- 同じexact診断入口50828は実1PASS/0.28秒、layout_svg=150035us、初frame_total=157182us、fragment後frame_total=4001us。cloud失敗を再現しておらず、DEBUG ON測定を通常性能や原因解消の証拠にしない。
- 最新live c07 checksはWindows/Linux成功、macOS失敗、他lint/供給網/CodeQL成功。新localのCIへ流用しない。

## 追加差分の自己レビュー

初稿のhelper直接呼び出し案は初回consumer検証が抜けるため撤回。元回帰の公開texture経路・実TextureHandle取得を維持し、製品修正を再検証する。Mock/timeout延長/ignore/閾値変更は無い。診断envは失敗後の別stepだけで、元テストの失敗結果を維持する。公開API・上流ソース・本人アプリ操作・hook迂回を追加していない。CI契約7件は成功、製品修正の対象focused2OS/AST/lintは進行中。全push/newHEAD CI/packaged/公開は未完了。

製品補正の再検証: native85122はloader11PASS/exit0、Linux73828は10PASS/PNG初回ロード1FAIL（失敗statusの内訳は旧panicに記録されず原因未確定）。失敗時にFailed理由とPendingを区別する診断だけを追加し、timeout/assertは変更せずLinux40070で11PASS/0.67秒/exit0。単独再成功を原因解消としない。AST92540/40757は各23PASS、strict impacted Clippy97322はexit0。fmt29427は折返しだけFAIL、公式formatter補正後再確認する。独立read-only再監査ではconsumer復元/新規キー保護/既存上限/oversized所有について採用可を得たが、全push/新HEAD品質は別工程。
