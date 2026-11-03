# v0.22.47 公開・検証記録

## 公開対象

[v0.22.47](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.47) は2026-11-02 22:52:20 UTCに一度だけ公開した。
公開製品のcommit Pは `6dabe8531e2b340630887967b63206b59ebc6de3`。
公開後の検証ツールC10は `3cab1fa28b78dad089365b2961fd6fc794fc1bb4`。
検証ツールの変更は[PR #416](https://github.com/HiroyukiFuruno/KatanA/pull/416)で通常マージした（`5ee12224d9aaa9c4d0c3152bd1a1b8eed0411da3`）。
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

時間・メモリ・応答の閾値は変更していない。
HTML ready 60秒、Office ready 15秒、close/Quit各5秒、case内heartbeat 200msを維持。
RSSとphysical footprintはcold→warm各196608 KiB、warm→final各65536 KiBを維持した。

| 7原本の実測最大値 | 結果 | 上限 |
| --- | ---: | ---: |
| HTML ready | 6627.756 ms | 60000 ms |
| Office ready | 5735.511 ms | 15000 ms |
| close | 950.192 ms | 5000 ms |
| Quit | 226.507 ms | 5000 ms |
| case内heartbeat gap（丸め上げ） | 172.383 ms | 200 ms |
| RSS cold→warm増加 | 173696 KiB | 196608 KiB |
| RSS warm→final増加 | 52464 KiB | 65536 KiB |
| physical cold→warm増加 | 128160.727 KiB | 196608 KiB |
| physical warm→final増加 | 52147.328 KiB | 65536 KiB |

[host acceptance](document-host/acceptance.json)、[bundle](document-host/observations/bundle.json)、7runとdownload receiptは正規producerで検証した。
各runのcontrollerはC10、実アプリのprovenanceはPであり、別々に照合している。
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

C10は13 suite/249 tests/skip0と通常pushを通過した。
[Ready CI 41687972078 attempt2](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41687972078/attempts/2)は8ジョブ成功、保護対象7チェックも成功。
attempt1は7成功・Mac Lintのみrunner割当て前キャンセルで、原因は未確定。正常な選択rerunでMac Lintを実行し成功した。
[Readiness 41687972102](https://github.com/HiroyukiFuruno/KatanA/actions/runs/41687972102)とexact C10向けCodeQLも成功した。
Windows CIがMac用Python/ShareDelete fixtureまで実行したとは扱わない。
最新レビューはP0/P1ゼロ、2threadは返信・resolve済み。
正常なworker履歴を失い誤拒否するP2は[独立Issue #417](https://github.com/HiroyukiFuruno/KatanA/issues/417)へ分離した。修正済みとは扱わず、合格rawは同じ必須ガードを通過したことを確認した。

## 正規画像と残る境界

[canonical画像](canonical-reference/sample-diagrams-top.png)と[capture receipt](canonical-reference/capture.json)はclean exact Pから公開後に実生成した。
1280×2400、PNG SHA256 `6b5ece9d9d3548e4f0250d9126535e10daf90768029f77e9b2ed46c8c3d26527`。
取得時刻は2026-11-02 22:53:11.920〜22:54:45.330925 UTC、KRRは0.4.23。
再生成入口は `scripts/screenshot/generate-sample-diagrams-reference.sh`。入力hash、実Git tree、geometry、interactionはreceiptに記録した。
C10へ再ラベルしていない。[canonical handoff #338](https://github.com/HiroyukiFuruno/KatanA/issues/338)で公開commitへの参照を共有する。

ZIP例外の既存期限は2026-11-07 00:00 UTCのまま。
[固定依存経路の評価](../../../docs/security/zip-0429-risk-assessment-2026-11-03.ja.md)と[Issue #354](https://github.com/HiroyukiFuruno/KatanA/issues/354)を参照し、延長・全programの安全・上流対応済みとは主張しない。
公開記録・archive準備と、記録PR自体のレビュー/CI/merge・その後の後片付けは区別し、後者の実績は[Issue #415](https://github.com/HiroyukiFuruno/KatanA/issues/415)で追跡する。
