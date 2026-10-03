# PR346 current review P1: 配布identityとDOCX証跡

レビュー対象HEADはa832b01b、review5401238907。指摘4173547522/4173547527へ対応する。公開・正常HTML close・全対象実機受入の完了とは別である。

## 配布する実バイナリの照合

preflightでの自己申告digest比較だけでは、後からActionsがビルドするバイナリを証明できない。新checkerをpublish jobのdownload/collect後、GitHub Release操作前へ配置した。同じ現ソース/lock/全受入証跡を再検査し、macOS ZIPのuniversal main/workerを両CPUの受入digestへ、Linux TARとWindows ZIPのmain/workerも各対象の受入digestへ照合する。さらに実DMGをreadonly mount、実MSIのembedded cabinetを7zzで読み、そのpayloadを同じdigestへ照合する。未知version・workspace version不一致・欠落・空・duplicate・symlink・hash不一致は拒否し、検査未導入toolも拒否する。ブートストラップでこの実配布検査を成功扱いしない。

回帰は旧workflowで公開前checkerが存在しないためRED、新workflowでGREEN。実ZIP/TAR payloadをmain/worker別に変える6条件は拒否、macOS両CPUの片側digest違いも拒否、duplicate/symlink/empty/missingのZIP/TARも拒否。version CLIの2回帰を含む8テストが成功した。preflightへ同じself-testを接続した。

実installerの抽出はfixture文字列では検証せず、以下の実成果物で確認した。

- 新v0.22.42 universal bundleはJOBS=2の既存package-macで成功、main/workerともx86_64+arm64、ad-hoc codesign --verify --deep --strict成功。実bundle由来診断DMGのreadonly検査が成功。main SHA `5ccc9a7634e8a13faa4e32eac762f33d957bab545b67dfa01a7ef63d537c368a`、worker SHA `5b8bd9a290620aeff7c3d942b9cdec09e307ff6b3a4d5e6246b35b4ec993d998`。ログ `tmp/kdv0510-dmg-identity-diagnostic-2026-10-03.log`。
- 公開済v0.22.41 Windows ZIP/MSIを取得。初版checkerでは7zzのMSI内cabinet自動展開を考慮せずmedia1.cab抽出が0bytesとなり失敗した。Compound/MSI/Cab型と一意WiX FileIDを確認し、実MSIからexe0/OfficeWorkerExeを直接取り出すよう修正。再実行exit0、ZIPとのmain/worker実SHA一致。ログ `tmp/published-v02241-msi-identity-2026-10-03.log`。ZIP SHA `97868335362d774f90af5dd0df02078caea92127be37235cc80cbed0188d078a`、MSI SHA `3653767b1350890d9d94847f7dcc1a3f7748c3b21e16bf38916ae7bea7f2a7d9`。これは抽出ロジックの実検証であり、新v0.22.42 Windowsの受入ではない。
- 元の未知version skipも独立レビューでP1として検出し、対象外CLIは非0、workspace.package version照合、workflowには対象version条件を追加して再レビューP0/P1なし。実機四対象の完全な受入JSONは未作成であり、新checkerの現root実行は証跡欠落でexit1（正しく公開を止める）。

## DOCXを必須Office集合へ追加

legal data-descriptor DOCX SHA `a1b7e22021218d314bc2d90c526d6d682981828b67cef6e61d8cb2a71ef5742a`を元6 XLSX/PPTXに追加し、DOCX/XLSX/PPTX全形式・fixture identity・packaged run・fidelityを必須とした。既存の基準・安全制限を緩和しない。

`test_office_requires_the_supplied_docx_in_addition_to_original_six`をa832b01bの旧validatorへ実行すると `AcceptanceEvidenceError not raised` でRED。ログ `tmp/docx-required-old-red-2026-10-03.log`。新validatorでは同テスト、DOCX hash/format/fidelity/packaged fixture digest拒否を含む41テストがGREEN、release gate17テストもGREEN。

## 未完了と追加性能証跡

新arm64 packaged startupはピークRSS234352KiB（約229MiB）、font27093388bytes、Office worker0、継続heartbeatと実main PID/path/SHA検査に合格。これはfresh Terms画面を含む空workspace検査であり、Office入力やclean-machine全対象の完了ではない。

同原本HTML DEBUG=true診断の初回はDOM1.537ms、JS21.585ms、CSS投影45.001ms、layout_svg28509.727ms、raster957.461ms、frame29517.876ms。再sessionでも初回layout_svg26009.805ms、再frameでは約26–27ms。JSではなく初回layout/SVG生成が遅延の大部分という実ログを既存KRR #95担当へ送った。正常close5秒は失敗のまま、強制終了を正常close扱いしない。KRR0.4.23公開は再確認404、公開KDV経由の取り込みと再受入は残る。

DOCX修正は0435368、配布identity修正はa54940ddへ関心事を分けて署名付き正式commitした。通常push・各thread reply/resolve・current全review再取得は後続。P2 manual target公開方針はユーザー判断待ちを維持する。独立95 fidelity、packaged Office/HTML、全CPU/OS、Actions配布、公開、Issue/OpenSpec後処理は未完了。
