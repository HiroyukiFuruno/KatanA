# 配布候補の独立検証（2026-10-23）

## 結論と対象

上流公開を待たず、現コードからmacOS・Windows・Linuxの配布候補を実生成した。以下の限定検査は完了したが、公開・全OS実機・clean-machine・本人最終受入の完了ではない。

- 対象HEAD: `1b7c4b0489449b05e8a14b717b4610f410362e12`（署名G）。製品ソースを固定し、生成前後のtree・両lock・tracked cleanを照合した。
- tree: `412ef8b89ea813f3a1e151df4584f68bfcb97925`。
- 本体lock SHA-256: `927fac6b5c168bf578c0e787ea7ea17f0bd0c079a9cd13557a4c7134836c51dc`。
- 撮影lock SHA-256: `a0f0abdedb6751225121368cb633c0a0238e8b262d42497b3e185c50f9c5b5d5`。
- registry固定KDV `0.5.12` / KRR `0.4.23`を維持。製品Rust・公開API・品質条件・依存版は変更していない。

## 実行結果と証明範囲

| 対象 | 実結果 | 証明しない範囲 |
| --- | --- | --- |
| macOS app・ZIP | 元`just package-mac` actual0。本体・workerの両方がarm64+x86_64、全4sliceのminOS 13.0、Info v0.22.42、ad-hoc署名deep/strict、元ZIP配布契約0 | 実macOS 13・Intel実起動・有償署名・公証 |
| macOS抽出物の基本起動 | ZIPから独立した設定で起動し元smoke actual0。実PID/path/SHA照合、UI heartbeat継続、peak RSS 244256KiB、owned font 27093388bytes、Office worker 0。終了後PID不在 | Terms承認後のidle workspace、実mainの文書操作・正常quit、physical footprint、clean-machine |
| macOS DMG | 元DMG生成0、`hdiutil verify` 0。read-only mountでmain/workerの実行権限・署名・version・ZIP元bundleとのbyte一致を検査し、所有mountをdetach完了 | 利用者のApplicationsへのインストール、Gatekeeperの本人許可 |
| Windows EXE・ZIP | Linux ARM/Rust1.95/cargo-xwin0.21.4から元locked workspace releaseを実linkして0。実PE2本がx86_64、トップレベルが正確に2ファイル、元Windows配布契約0 | Windows実機実行、MSI、cold/warm、全配布受入 |
| Linux ELF・tar.gz | ARM-native Rust1.95→`x86_64-unknown-linux-gnu`の実release build0、元packageと配布契約0。2本ともELF x86_64、archive内0755。mainの必要GLIBC versionに2.39を実測 | Ubuntu公式runnerでのbuild、Linux x86_64実機起動・clean-machine |

macOSの基本起動はfresh設定のTerms前画面である。本人の設定・承認を複製せず、同意操作もしていない。smokeの終了処理は所有PIDの停止であり、アプリの通常quit成功とは扱わない。

## ZIP同梱Office workerの原本受入

ZIP抽出物のworker SHAは`4ecc684f9e0b570a6cff676ecac0914f14df9cdc3b48f08e85be792105c514cf`。既存撮影runner SHA `2208feac65df2731b8666e2cb284dcda67f152682265f12cd6c3fbebadf78953`へこの実workerを指定し、元request・通常診断OFF・期限・予算を変更せず検査した。

- Office5件を2回、各21stepとharness quitまでactual0。終了時のsession、worker、frame、texture、cache、artifact各保持数は0。
- 全文書close後の親RSS増分は168896 / 196592KiBで元196608KiB以内。ただし後者の余裕は16KiBだけであり、安定解消や一般的なリーク不存在とは主張しない。表示中の記録点では2回目の増分が196640KiBであり、close後予算と混同しない。連続peakの測定でもない。
- HTML/PPTXのcold1回＋warm10回はactual0、HTML/documentの観測は各11、終了時の各保持数0。RSS増分154128 / -6000KiBで元196608 / 65536KiB以内。
- これはheadless host＋実同梱sidecarの検査であり、配布mainの全文書・physical footprint・独立4軸95点の代用ではない。

## 失敗の回収と修復・限界

- macOS初回はnative bundle生成後、所有buildだけを中断してactual130を保持した。資源を確認して並列数を4へ調整後、同じ入口・targetで再開して0。完了済みnative buildは2.42秒のcompiled cache利用だった。テスト結果の再利用や速度改善の比較証明ではない。
- 共有worktree内のZIP抽出はmainだけ0755→0600になり、元smokeがactual1で拒否した。archiveの両modeは0755、byte/SHAは一致していた。同一ZIPを`/private/tmp`へ新展開すると、chmodせず両0755・署名・元smokeが0。CI同様に作業tree外へ展開した結果であり、共有領域で権限を変更する主体は未確定。Avast等へ原因を断定しない。
- Windows初回のactual101は`Userenv.lib` / `Advapi32.lib`の名前解決失敗だった。元SDKにある`UserEnv.Lib` / `AdvAPI32.Lib`とのcase差を実確認した。共有SDKはreadonlyで複製し、[xwinのmap形式](https://github.com/Jake-Shadle/xwin/blob/main/docs/example-map.toml)に沿った2つの追加aliasだけを隔離cacheへ適用した。新SDK取得・EULA代行・製品コード変更はない。
- 実x86_64 Windows API callerのtiny probeは旧cacheでlink exit1、補正cacheで0。PE importに`ADVAPI32.dll:OpenProcessToken`と`USERENV.dll:CreateEnvironmentBlock`が残り、各alias/canonical SHAは一致、共有元の要求alias不在と元SHA不変を検証した。続く製品release linkはactual0。missing libraryの無視、defaultlib除外、検査skipによる製品合格ではない。
- Linuxのamd64 QEMU下cargoは進行を調べた後、所有containerだけを終了してactual137/OOM=falseを保持した。中断を製品のbuild成功や製品障害の確定とはしない。muslへ変更せず、ARM-nativeから同じGNU targetへcross-buildして0を回収した。
- Linux起動検査の初回actual1は検証imageの`libXcursor`欠落だった。標準X11/Mesa runtimeを追加したimageで同じtar・元smokeを再実行すると、heartbeat生成後のprocess image同一性検査でactual1となった。QEMU環境の不一致を保持し、guardを緩めて合格にはしない。native Linux起動は未検証である。

## 成果物SHA-256

| 成果物 | SHA-256 |
| --- | --- |
| macOS ZIP | `4806a696faf06020b0fe60a094be7820d180c5034fca6c197bc798dc2a47f8cd` |
| macOS DMG | `e42c77bce0671773fab9f67ebc53b4de0aab1560736830ba24eede9b15944858` |
| Windows ZIP | `f5c2d8239e723c62214108d6eefdaf8e9af1dd99d883c333f848ac8697edd05f` |
| Linux tar.gz | `82eaa5ff384d1162483590bca73f698710aaeed7f8a5dd39987e370211a27427` |

成果物はローカル候補であり、GitHub Releaseへの公開・CI provenance取得ではない。Windows MSIは生成しておらず、全5配布asset完成とも扱わない。

## 物理証跡

- macOS: `tmp/oct23-current-package-mac-jobs1-interrupted.log`（中断130）、`tmp/oct23-current-package-mac.log`、`tmp/oct23-macos-archive-acceptance.log`、`tmp/oct23-macos-private-extraction-startup.log`、`tmp/oct23-macos-dmg-acceptance.log`、`tmp/oct23-macos-dmg-content-check.log`。
- sidecar: `tmp/oct23-packaged-sidecar-cycles.log`、`tmp/oct23-packaged-sidecar-cycles/`。原本の私的内容は公開文書へ転載しない。
- Windows: `tmp/oct23-current-windows-package-with-git.log`（旧失敗）、`tmp/oct23-current-windows-package-casefix.log`、`tmp/oct23-current-windows-package-casefix-acceptance.log`、`tmp/oct23-xwin-casefix-map.toml`、`tmp/oct23-xwin-casefix-link-probe.log`。
- Linux: `tmp/oct23-current-linux-package.log`（QEMU中断）、`tmp/oct23-current-linux-cross-package.log`、`tmp/oct23-linux-elf-runtime-contract.log`、`tmp/oct23-linux-emulated-startup.log`、`tmp/oct23-linux-emulated-startup-with-deps.log`、`tmp/oct23-linux-package-retained/startup-evidence/`。
- 4候補の保持hash: `tmp/oct23-packaged-artifacts.sha256`。全rawは成功・失敗を分けて保持する。

## 未完の公開条件

KDV #69の安全な公開依存採用、通常pushと新HEAD全review/個別reply・resolve、Ready後の必須CI、Windows/MSI・Intel/Linux実機、実mainの全memory/文書・clean-machine・本人受入、全5asset/checksumと公開後処理は未完である。Draft重CI、hook迂回、新worktree、master編集、stash、上流実装、本人app操作は行っていない。
