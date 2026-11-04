# v0.22.47 公開・検証記録

## 公開対象

[v0.22.47](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.47) は2026-11-02 22:52:20 UTCに一度だけ公開した。
公開製品のcommit Pは `6dabe8531e2b340630887967b63206b59ebc6de3`。
公開後の文書受入に用いた検証ツールC11は `c569d8c9b671bf59882f27fb6103d707a4203573`。
先行するC10の検査訂正は[PR #416](https://github.com/HiroyukiFuruno/KatanA/pull/416)で通常マージした（`5ee12224d9aaa9c4d0c3152bd1a1b8eed0411da3`）。
C11では配布物5件とchecksums.txtを含む公開identity6件をそれぞれ厳密に照合する。全descriptor・実API・3manifestの照合を保持し、この訂正と新規受入記録を本記録PRで検証する。
公開tag・アプリ・5配布payloadは変更していない。

[公開workflow 41531951434](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41531951434/attempts/1)は対象8ジョブ成功、任意のWindows Server 2022比較1ジョブはskip。
実5payloadのAPIサイズ・SHA256・checksums・3manifest・attestation subject/identityを照合した。

| payload | bytes | SHA256 |
| --- | ---: | --- |
| macOS DMG | 223147967 | `32f7d255996860a3d6d2bce1150abe13c454aa7e83e4dfbc9a1bd958859121e5` |
| macOS ZIP | 223459963 | `d8ed3e022b4705cd3f32c3768d4189709a699d55672e5927f9ae5a307e4fee83` |
| Linux tar.gz | 120658604 | `d160139b5c2e54a16fd943eb4d167e6020d72a79b37fdfe8e15b1aaae517e6d7` |
| Windows MSI | 109203456 | `701fc7b788786c6ceccb9140e1045a592586597e7096a76fd7d9508bbc0ed701` |
| Windows ZIP | 105875520 | `7326575927ec629ec3f7ada615bb2c02fa77878d76102a20c60ade52e383dfc7` |

[公開identity検査 41541059501](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41541059501)のidentityジョブも成功。
このrunのReadinessジョブはskipであり、全Readiness実行の証拠とは区別する。
取得したidentity ZIPは9596 bytes、SHA256 `7592f2238ed9aab9833f8153b3f09bec53df5fedbea5b1eff46c7e874d96ae9a`。
rootのpublication identityと3manifestのcontroller identityはPのまま保持する。

## 公開版の文書受入

公開Mac ZIPのarm64実行で、原本7種類それぞれHTMLとの10 mixed cycles、合計140ケースを新規に実行した。
形式別の操作、HTML scroll/reload、実worker、最終idle、main/workerの同一性、正常終了、終了後OS上の不在を検証した。
失敗した過去の試行は不採用として保存し、別controllerへの付け替えや後付け救済はしていない。
C11の原本06初回はOSのメモリ値を正の値として取得できず拒否した。不明値はnullのまま保存し、同一C11・同一閾値で新規に全20ケースを取り直した正式合格runを採用した。

時間・メモリ・応答の閾値は変更していない。
HTML ready 60秒、Office ready 15秒、close/Quit各5秒、case内heartbeat 200msを維持。
RSSとphysical footprintはcold→warm各196608 KiB、warm→final各65536 KiBを維持した。

| 7原本の実測最大値 | 結果 | 上限 |
| --- | ---: | ---: |
| HTML ready | 2276.947 ms | 60000 ms |
| Office ready | 5599.900 ms | 15000 ms |
| close | 923.527 ms | 5000 ms |
| Quit | 233.809 ms | 5000 ms |
| case内heartbeat gap（丸め上げ） | 172.304 ms | 200 ms |
| RSS cold→warm増加 | 171472 KiB | 196608 KiB |
| RSS warm→final増加 | 39888 KiB | 65536 KiB |
| physical cold→warm増加 | 152144.727 KiB | 196608 KiB |
| physical warm→final増加 | 33763.023 KiB | 65536 KiB |

[host acceptance](document-host/acceptance.json)、[bundle](document-host/observations/bundle.json)、7runとdownload receiptは正規producerで検証した。
各runのcontrollerはC11、実アプリのprovenanceはPであり、別々に照合している。
Windows/Linuxのpackaged起動成功は文書操作受入の代用にしない。他targetの文書QAは未実施。
Chrome HTML比較とOffice独立95点の評価は `not_evaluated`、score/measurementはnullのまま保持した。

公開アプリでグループ名入力・空欄Enter・作成後close、Cancel/Esc、色表示、同一「名前＋色」の作成/rename/recolor拒否を確認した。
名前任意、同名異色・異名同色・空名異色は許容し、4グループの保存と別process再起動での復元を確認した。
通常settings/workspaceのSHA256・mtime・bytesは検証前後で一致した。

## CIの実績と計測範囲

[PR #405のReady CI](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863454)は全8ジョブ、[Readiness](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863578)も成功。
[公開元Pのmaster CI](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41515109936)も全8ジョブ成功。
文書面coverage 100%・未カバー0はその面の結果であり、workspace全体100%とは扱わない。

| 測定 | 値 | 範囲・未証明部分 |
| --- | --- | --- |
| Windows cold/warm | 1446.827 / 963.324秒 | commandのみ。setup・cache転送は含まない。compiled targetはmiss→hit、generic cache_hitは両方false。 |
| macOS normal/coverage | 532.460 / 785.008秒 | PRのcommand合計。coverageは4phase。 |
| macOS inventory | common2430 / normal-only10 / coverage-only0 | inventory不完全、execution equivalence=false、reuse未認可。 |
| P master coverage | 1521.311秒 | 4phase合計。PRの時間と環境差を区別する。 |

[Windows計測run](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863247)の条件・入力identity・artifact digestを照合した。
計測結果から完全な検査同等性、汎用skip/reuse、CI全体の高速化を証明済みとはしない。
補足の全repo Markdown検査は当時の未変更197ファイルに858診断で失敗した。
重複0は歴史的なWindows stdin修正5ファイルとの比較だけであり、PR #405全体や現在の全体lint成功の主張ではない。変更Markdown3ファイルのscoped検査は成功した。

### 計測artifactへの固定参照

PR計測2runはattempt 1、API headは `2fe7c41d2e11626a7f7bb7e73eee33cc49ce2309`、測定JSONに記録された実際のPR merge checkoutは `16a63caa9261dbfd38fd77b4907b4855fe1c57f7`。
master計測はattempt 1、API headと実checkoutともP。次の実ZIPのAPIサイズ・SHA256と内部測定JSONを照合した。

| run / artifact | bytes | ZIP SHA256 |
| --- | ---: | --- |
| [41501863247 / windows-cache-cold / 13171515517](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863247/artifacts/13171515517) | 3730 | `646b299936ee4b12fd379f0229d673cd7130b01fd06ec7186dfcaa34966d9b10` |
| [41501863247 / windows-cache-warm / 13171463773](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863247/artifacts/13171463773) | 3731 | `8718ce53a1fdb20d0948db8e037728abe73b2b85b5a70ce0db0bfad55c6435f7` |
| [41501863247 / windows-cache-comparison / 13171388639](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863247/artifacts/13171388639) | 1804 | `ea8c00c3056f46768826a0c09fe8f8413aa2454e4d0f4c641705ea6316af3a1f` |
| [41501863454 / test-inventory-macOS / 13172670417](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41501863454/artifacts/13172670417) | 249380 | `a78623e7c68c1faac41a7df95225f4862b25bc5e73a274355e0b1760b5fb7540` |
| [41515109936 / test-inventory-macOS / 13180994258](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41515109936/artifacts/13180994258) | 249209 | `30d4728f34912b86697f4a3ee03ca7353efe4304ee5456bbbfe5d9eb89b85d12` |

Windowsのmember SHA256はcold.json=`fb85c7d448a9f7d7c7e6f5e8357ceaa8e35b545a71850dca77daacbdb8bca9f5`、warm.json=`f9a9b183634f6f2ec21745da0316652e44877c1c616117610fbc6b082777696c`、比較JSON=`da0307751c2a222e62b33d9af36c5f2843fae6f21a7a362edc7c906bd00f64c1`。
macOS各ZIPにはnormal.json・coverage4phaseの各JSON・comparison.jsonを含む。comparison.jsonのSHA256はPR=`56a0f167055e88cdad858f409b5db8b6d97555e4e1dffec624e3fdc2f9b87427`、master=`0cb058e2bfe8afa631aee0c0d1fa51393b19ee389728f210785afa9f25c90371`。
公開アプリ5payloadのdigestと、これらCI計測artifactのdigestは別物として保持する。

C10は13 suite/249 tests/skip0と通常pushを通過した。
C11も対象13 suite/249 tests/skip0、独立Highレビュー、実際の公開記録を用いた旧RED→新GREENを通過した。C11の正常push・本記録PRのreview/CI/merge実績は#415で追跡する。
[Ready CI 41687972078 attempt2](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41687972078/attempts/2)は8ジョブ成功、保護対象7チェックも成功。
attempt1は7成功・Mac Lintのみrunner割当て前キャンセルで、原因は未確定。正常な選択rerunでMac Lintを実行し成功した。
[Readiness 41687972102](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41687972102)とexact C10向けCodeQLも成功した。
Windows CIがMac用Python/ShareDelete fixtureまで実行したとは扱わない。
C10の最新レビューはP0/P1ゼロ、2threadは返信・resolve済み。
正常なworker履歴を失い誤拒否するP2は[独立Issue #417](https://github.com/HiroyukiFuruno/KatanA/issues/417)へ分離した。修正済みとは扱わず、合格rawは同じ必須ガードを通過したことを確認した。

## 正規画像と残る境界

[canonical画像](canonical-reference/sample-diagrams-top.png)と[capture receipt](canonical-reference/capture.json)はclean exact Pから公開後に実生成した。
1280×2400、PNG SHA256 `6b5ece9d9d3548e4f0250d9126535e10daf90768029f77e9b2ed46c8c3d26527`。
取得時刻は2026-11-02 22:53:11.920〜22:54:45.330925 UTC、KRRは0.4.23。
再生成入口は `scripts/screenshot/generate-sample-diagrams-reference.sh`。入力hash、実Git tree、geometry、interactionはreceiptに記録した。
C10へ再ラベルしていない。[canonical handoff #338](https://github.com/HiroyukiFuruno/KatanA/issues/338#issuecomment-6615644808)で公開commitへの固定リンクを共有した。

ZIP例外の既存期限は2026-11-07 00:00 UTCのまま。
[固定依存経路の評価](../../../docs/security/zip-0429-risk-assessment-2026-11-03.ja.md)と[Issue #354](https://github.com/HiroyukiFuruno/KatanA/issues/354)を参照し、延長・全programの安全・上流対応済みとは主張しない。
[OpenSpecの記録](../../../openspec/changes/archive/2026-11-04-v0-22-47-katana-contained-fixes/tasks.md)へアーカイブし、42タスクのID・本文と同期済み8main specの不変を照合した。
記録PR自体のレビュー/CI/merge・その後の後片付けの実績は[Issue #415](https://github.com/HiroyukiFuruno/KatanA/issues/415)で追跡する。
