# 更新元整理の実ホスト限定

対象: PR346 HEAD04e596f0のmacOS CI37491322251、job112364501947。

元HTML初frame検査は2007msで失敗し、idle=false/generation=Noneを保持した。正常取得された失敗sample335920bytesにはHTML layout/text shaping中のworkerと、65thread中59本のUpdateCleanupOps→ProcessService::status→wait4を確認した。診断childsampleは初回frame窓と重ならず、別processの約137ms成功は元失敗の修復証明ではない。詳細rawは `tmp/04e596f0-current-ci-audit/analysis.md` に保存した。

## 所有修正

`KatanaApp::new` は実hostだけでなく文書検査用fixtureでも使用される。そこで毎生成OS更新元整理をspawnしていた呼出しを、実native AppCreatorの文書アプリ生成直前へ移した。既存coreのbrew unpin/untap script、非同期処理、エラー方針、各OSの実装は変更していない。文書fixtureの生成はこの副作用を持たなくなる。coverage/test条件や2秒基準は変更しない。

## 検証

- 変更前のsource契約回帰84951: `tmp/native-cleanup-contract-red.log`、actual101、constructorのcleanup呼出しに対する1assertFAIL。
- 変更後のnative全18テスト80361: `tmp/native-cleanup-contract-green.log`、actual0。契約はconstructorに呼出しがなく、native AppCreatorに一箇所・生成直前に残ることを検査する。
- 公式AST/impacted strictClippy/fmt26510: `tmp/native-cleanup-current-static.log`、actual0、AST23PASS。`git diff --check` actual0。
- 通常署名付きcommit `bc78503a`。URL閉鎖取消`ad264741`とbinary形式移行`3b4e2770`とは関心を分離した。

この修正は検査fixtureによるOS副作用の除去を証明するもので、元2秒超過の因果確定や全CI成功の証明ではない。新HEADで通常push hook・全CI・coverage・packaged受入を検証する。本人アプリの操作・Terms承認は行っていない。
