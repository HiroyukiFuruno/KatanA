# c07 current review追加3件の修正検証

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
