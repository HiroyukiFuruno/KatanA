# Office終了後メモリの診断observer

## 最新graphでの通常実動作検証

最新ユーザー指示に従い、KDV実装公開と実動作受入を分けてKatanA側の通常検証を実施した。現root lock SHA57924ea04f881b42b5ca26f69c27dea09b1ebc25ffe1e6a92b267d450fb84f63、runner lock SHAb48fe0cdba8c396128fdf1b4d798801f9c5eedbfd9bd51b339fe5dd22bb93930。fresh locked build53690 actualexit0、worker SHA6b0676b83f24500e5f9af766c60bcec14643ba09c128cd86f698f21b70a2c72a、runner SHA838c101221a8b9bd10f3969cffcf68da8c25deb5550940e0b1cb44ac6e45fafc。原Office request SHAはf58b929fab7430afc4eb5893f49bccd0d7a7efd6a8cf2fddc29339c7a3641d0cのまま。

診断OFF・重buildを重ねない専用窓でOffice5を二回実行。45841/35029ともactualexit0、全21steps/quit、終了後の保持資源全0。RSS105056→300768KiB（delta195712）と105184→300608KiB（delta195424）で元上限196608を満たした。余裕896/1184KiBのみで安定解消とは断定しない。PPTX初回frame最大4.599秒で元15秒以内だが全format2秒未満とは報告しない。rawはlocal tmp/release-final-office5-normal-{1,2}-2026-10-06.log。

5070の原loom+HTML cold1/warm10もactualexit0、全10steps/quit、観測HTML/document各11、終了保持資源全0。cold104144→260720KiB（delta156576）、warm260720→244704KiB（delta-16016）、元cold196608/warm65536上限不変。rawはlocal tmp/release-final-loom-warm-2026-10-06.log。既存KDV担当へ窓終了と結果を引き継いだ。個人入力path/raw/画像は公開しない。

現graphの公式lint/fmt/供給網と実worker付き候補回帰8件はactualexit0。コメント構文回帰8件・正式AST23件もactualexit0で、説明コメントのみ最新日本語指示へ整合し識別子/文字列制約を維持した。過去の言語方針再質問待ちは撤回した。通常commit/push、新HEAD全gate/coverage/current review、独立各95点、全packaged/native・公開後処理は別の残DoDである。

## 最新の原本測定と次の限定修正候補

locked release build13063はactualexit0。worker SHA3914472e4d9ceb8a914375374a1f30b33e0d4bb40d14abfd06fe4945d5c1bfe7、runner SHAbbb797d48cbf2fd35f19251945dde9639cc0648a2de73ea8b60b25d8b70e5069。root lock b8711c3ae2162dcafb4be095985356b0175082f86bda4ff9f0a2a23a0cef6f13、runner lock aae4111b049801dfe7df0987d66a0094def70225829a9ac1f3b4f0d183830833。原Office5 request SHA f58b929fab7430afc4eb5893f49bccd0d7a7efd6a8cf2fddc29339c7a3641d0cを維持した。

通常flag-OFF受入99302はactualexit1。cold104880→正常close後302640KiB、delta197760KiBが元上限196608を超過した。idle countersは0だがstep20のRSS assertionはFAILでstep21 quitへ到達していない。KDV0.5.12の公開/採用やobserver追加で解消したとは扱わない。local rawはtmp/office-five-observer-disabled-2026-10-06.log。

診断ON84495は同binaryでactualexit0、全21steps/quitを実行し親PID82237の26manifestを取得。vmmap/heap全actualexit0・timeout/output超過false、終了後親不在を確認。RSS105072→284992KiB、delta179920。ただしdiagnostic実行が時刻・malloc回収へ介入するため、通常受入のFAILをこのPASSで置換しない。rawはtmp/office-five-owned-memory-diagnostics-2026-10-06.logと同名directory。原本の個人情報を含むraw/画像/絶対入力pathは公開しない。

同親baseline-close→final assertionで、vmmapのMalloc Large (empty) residentは24.5→175.5MiB、Malloc Small (empty)は112KiB→11.9MiB。heap liveは56848256→59270176bytes（差2421920bytes、約2.31MiB）。観測した支配的な保持は解放済みmallocのresidentである。共有library residentの全量をprocess RSSへ足さない。観測OFFでの正確な内訳やKDVの原因除外は未証明。

Apple公式libmalloc header/実装とlocal CLT SDKでmalloc_zone_pressure_relief(NULL,0)の契約（全zone・最大best-effort回収・返却0も合法）を確認した。候補は最終document-previewが実際に削除され、open documentsが空になる遷移だけに限定する。毎action、HTML-only、残存/pinned文書、invalidclose、反復emptyでは呼ばず、二重cleanupも一回にする。公開API、新allocator、test用product port、固定待機、元閾値/描画仕様は変更しない。通常flag-OFF原本の実改善、live buffer保持、実app caller、全gateを満たすまでは採用・完了扱いしない。

候補初稿はpure判定と無assert native呼出しだけだったため主担当は不採用として補強した。最終版は8条件、実macOS native呼出しを挟んだlive Vec pointer/length/content保持、実KatanaAppのlast document cleanup・二重cleanup・pinned/残存dirty内容保持・HTML-only cleanupを検証する。新allowなし・pub(super)・日本語SAFETY、native ABIはSDK size_t/opaque zoneと照合。主担当の実Office worker付きcanonical90448 actualexit0、8pass/失敗0/ignore0（lib1061filter）。local rawはtmp/closed-preview-memory-main-canonical-2026-10-06.log。全gateや原本RSS改善とは独立のfocused証明であり、64906のlocked release再build・通常原本再測定を継続する。

64906はactualexit0。worker SHAは3914472eのまま、runner SHAだけbc8570c5f1d77fc368ae47456b9a4e7a42ef8bef424a25172e11bc3a5ce8520fへ変更、両lock/requestは同一。候補の通常診断OFF Office5は35960と14095の2回ともactualexit0・全21steps/quit・idle資源0。RSSは104912→301088KiB（delta196176）と104864→299296KiB（delta194432）、元196608KiB上限を満たした。上限余裕は432/2176KiBのみで、観測OFF旧失敗との小差を安定解消・全保持原因の解消へ誇張しない。通常closeは初回max0.097秒/反復max0.103秒、初回frameの全format元15秒基準・close5秒を維持。local rawはtmp/closed-preview-memory-office-five-normal-1-2026-10-06.logと同-2。既存loom cold1+warm10複合反復の元cold196608/warm65536基準、全gate/current review/packaged各95点を継続する。

70236の原loom+HTML cold1/warm10複合はactualexit0・全10steps/quit、観測HTML/document各11、idle全0。cold104032→258432KiB（delta154400）、warm258432→252432（delta-6000）、元cold196608/warm65536上限を維持。rawはtmp/closed-preview-memory-loom-warm-cycle-2026-10-06.log。

main48046はall-target strictClippy/fmt成功後、正式ASTで20pass/3fail、actualexit101。observer直Command::new、コメント形式、日本語禁止を検出した。observerは既存ProcessService::create_commandへ補正し50851公式locked release契約54件・all-target strictClippy・fmt actualexit0。候補のコメント形式だけ補正してロジックは不変。最新ユーザーの日本語コメント指示と既存日本語禁止チェックが矛盾するため、ユーザーへ適用方針を確認し、英語化/言語チェックの緩和は独断で行わない。通常全gate、正式commit/push、newHEAD review、packaged/95点/公開は未完了である。

次heartbeatの独立回帰では、公式just test-integration88290 actualexit0/fixture8pass、実worker付き全UI lib9340 actualexit0/1067pass/0fail/既存ignore2/0filter（52.53秒）を確認した。rawはtmp/comment-policy-pending-fixture-regression-2026-10-06.log（SHA1f607085312438a5eb300797b0bc9e752490b1e08d2752ae69d58a67354f2390）と同ui（SHAb9168947be6a4f5279a764cb6f846f191cbc2c672c56d20d4def7bf3f2f9ad39）。コメント方針未回答のまま規約を変更しておらず、AST言語不整合・全coverage/全platform/current新HEAD review・公開DoDは未完了。

## 目的と非完了条件

旧5f9の通常原本Office5受入は2回とも元RSS上限196608KiBを超えて失敗した。idle資源counter0は漏れの原因も受入成功も証明しない。既存KDV #59担当からKatanA #339 comment5998964487へ引き継がれた、同じ親runner PIDのcold/各正常close直後のmalloc割当・VM内訳を取得する。診断追加やKDV0.5.12公開をRSS修正成功へ読み替えない。

## 実装境界

変更はscripts/screenshot/src/main.rs、executor_harness.rs、新memory_observer.rsのみ。製品API、描画、原request、正常close期限、元RSS/品質基準は不変。任意 --memory-diagnostics は既定falseで、通常実行では診断コマンドとファイル処理を行わない。enabledはmacOSだけ、rootをcanonicalizeし、std::process::id()の親自身に固定argv /usr/bin/vmmap -summary PID と /usr/bin/heap -s PIDを使用する。shellや外部PIDを受け付けない。

既存RecordRuntimeSnapshot、成功したCloseActiveDocument/CloseAllDocuments、最終RSS assertion前に接続する。stdout/stderrは直接ファイルへ送って親に大量bufferを保持しない。create_newで既存file/symlinkの上書きを拒否。poll中の合計8MiB/15秒安全上限でowned diagnostic childをkill+waitし、metadata/try_wait失敗でも回収する。時刻エラーを隠さない。manifestはdiagnostic_only、canonical root、step/phase、親PID、固定command/argv、時刻、exit status、timeout/出力超過を記録する。診断失敗は通常成功へfallbackしない。

## 主担当レビューと検証

初稿の50件成功だけでは、終了後のみの出力量判定、monitor失敗時のchild回収、実mac probe不足を証明できないため統合しなかった。既存childへ同じ編集境界で補正を依頼し、主担当が最終差分と実probe rawを照合した。補正後childは54件test/strict Clippy exit0を報告。主担当の公式CIと同じlocked release screenshot契約検査1164もexit0、54pass/失敗0/ignore0。rawはtmp/memory-observer-canonical-release-contracts-2026-10-06.log。主担当のrelease all-target strict Clippy68303とfmtもexit0、rawはtmp/memory-observer-canonical-clippy-2026-10-06.log。基準/除外/受入assertは変更していない。

実mac probeは親test PID72806だけを対象とし、両manifestで固定argv/exit0/diagnostic_only=true/上限未超過を確認。rawはlocal /private/var/folders/ql/4640yx8s22zg367pjjld7yc00000gn/T/katana-memory-observer-owned-probe-1791220721960284000に保存。これは検査processの観測能力だけを示し、原本Office runnerの保持内訳やRSS改善ではない。

KDV0.5.12固定採用後の公式実worker/HTML関連8件、host契約24件/実process-group7件、供給網4分類は成功。変更後の原本診断、通常flag-off受入、全gate、正式commit/push/current review、packaged全OS/独立各95点/公開後処理は未完了。

通常commit21579 exit0で依存差分000a2869、別commit63596 exit0でobserver9401a5b0を正式履歴へ統合した。rawはtmp/kdv0512-adoption-source-commit-2026-10-06.logとtmp/memory-observer-source-commit-2026-10-06.log。新release runner/workerを13063でlocked build中であり、旧binaryの原本結果を新sourceへ流用しない。通常push、新HEAD review、全gate/原本診断・受入・公開DoDは後続として継続する。
