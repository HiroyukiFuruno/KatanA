# v0.22.42 3機能の公開証跡

- Release: [v0.22.42](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.42)
- 公開時刻: 2026-10-28T11:27:18Z
- 配布元M: `8c89a1544e9ac693d5ba388792a843baa55c9363`
- 実取得: `evidence/v0.22.42-public-download-40672472894.json`（通常署名commit d580b88b）。全5件/checksum/size/URL/本体worker/3producer一致、実行56766 exit0。
- source CI: reviewed H a6584bce / actual CI checkout2aa64b46 / M8c89は同一tree、CI40660526599全8・Readiness40660526626・current Codex未対応0。M push CI40672234693成功。named-test-mapping.jsonに8件の各OS実raw行を記録。
- 正式40672472894/attempt1のpreflight/3build/両Mac native/元Windows200ms×10/Publish成功。cleanup installerだけ失敗してrun全体failure。この失敗を成功と偽らず、root台帳oct28_postpublication_cleanup_recoveryとoct25_public_assets_and_cleanupで独立修復を継続。
- Office全体95点/PPTX初回15秒の未達や未実施本人QAをこの3機能の成功へ読み替えない。umbrellaはactiveのまま。

| 公開ファイル | bytes | SHA256 |
| --- | ---: | --- |
| KatanA-Desktop-0.22.42.dmg | 222701573 | `757a5a503170d351c91f436496b73c3e3c800f1b86614cc2ef3ec34f0df889fb` |
| KatanA-linux-x86_64.tar.gz | 120835389 | `53024e279495e0e6607273cfbd7633ef16c6e11784c37449c35ac556cfecf555` |
| KatanA-macOS.zip | 222997316 | `1b29b8ab75cf415395c53176761c6508b5003fd9eda53174b03b4bc28153beb7` |
| KatanA-windows-x86_64.msi | 108933120 | `68cf88123cc7d8964234baa1d65044b1dbd66bfe96522bf1368ad1de61b2cf58` |
| KatanA-windows-x86_64.zip | 105624933 | `efbd6ae00eda54dd1309cf1c20b738371261745f0650007da08dd53d0125f0e7` |

spec syncは3追加Requirement/7Scenariosのみ。既存main全文prefixとlocal asset既存1Requirement/4Scenariosの1222bytes/SHA944e2fdf5ed10de09f6e1a99022489ee1420c6c1cc852571c716e1fd3809111cを保持する。

後処理docs PRのreview/CI/mergeと安全なcleanupはこのchildのsource実装deliveryとは分離し、root台帳で未完のまま実行する。公開済みtag/assetsを再作成しない。

修正版検証経路でも78200 exit0でfresh全5assetを再取得した。`evidence/v0.22.42-public-frozen-controller-47942653.json` は同じ公開asset ID/size/SHA256・配布元M・全producerに一致する。検証コードHEAD47942653と配布元Mを混同せず、`evidence/v0.22.42-postpublication-controller-47942653.json` に分離して記録する。
