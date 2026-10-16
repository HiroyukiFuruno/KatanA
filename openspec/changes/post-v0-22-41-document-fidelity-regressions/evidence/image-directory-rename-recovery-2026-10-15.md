# 画像監視ディレクトリ移動の回復

## 対象と未完了条件

PR346のP2 comment4268050392 / thread PRRT_kwDORm09y86sNe-Qを、既存Issue348の画像監視回復として対応する。KDV/KRRの実装は変更しない。公開HEADは2a7a7107738d9918560a260403326d15749e5d4a、限定sourceはdd65b4c9へ通常commitした。pushはまだ行っておらず、全OS・配布・公開・Issue全体の完了を意味しない。

## 実再現

- session22387: 旧製品source、実RecommendedWatcherへcanonicalな移動元・子・無関係directoryを登録し、実filesystem renameを実行。終了101、0PASS/1FAIL、0.15秒。移動元がwatched_dirsへ残るassertionで失敗した。
- 生ログ: tmp/image-directory-rename-native-red-2026-10-15.log。先行59905はfixtureのイベント取得Timeoutであり、製品不具合のREDとして扱わない。
- notify8.2.0の公開registry source inotify.rsはMOVE_SELFをModify(Name(From))へ分類する。macOS上のnative再現はLinux kernelの実動作証明ではなく、両者を混同しない。

## 修復契約

Removeに加え、Rename From/Any/Bothで失われた登録directoryと子を既存detach/Failed通知へ戻す。Bothの二番目は移動先なので解除対象へ含めない。無関係directory、To、通常Modify、画像ファイルのRemoveは保持する。所有者generationと既存bounded再登録・実解除時のoverflow補完を維持する。新worker、公開外部API、mock、testport、allow、ignore、timeout延長は追加しない。

## 検証

- session91142: rename分類とnative回帰3PASS、終了0、0.40秒。
- session62221: local_image_loader全53PASS、終了0、2.72秒。Both(old,new)の実登録回帰を含む。
- 関数分離後session63767: 同53PASS、終了0、2.75秒。
- 最終source session96602: 同53PASS、終了0、2.69秒。
- 正式AST95061は22PASS/1FAIL（テスト診断保持上限のliteral8）→const抽出後99098は23PASS。最終source42873は23PASS、終了0、23.53秒。
- 最終source41098: katana-ui全target/test-inclusive・external-fixture-acceptance付きstrictClippy終了0。fmt-check/diff-check終了0。
- readonly独立レビューはP0/P1無し。実native fixtureは追加recursive root監視を含むため、production Coordinatorのsubscriptionだけによる全3OSイベント配信証明ではない。
- 自己レビューは新関数30行以内・対象files200行以内、owner/generation/既存retry/overflow不変、実RED、外部API/serde/依存変更無しを確認。self-review旧英語コメント指示ではなく、最新本人・docs/coding-rulesの日本語WHY規約を適用した。
- 通常commit78736は終了0、source dd65b4c9。関心別push・個別reply/resolve・fresh reviewは未完。

再登録後の全packaged原本入力・全OS・各95点・リリース成果物受入は、この限定検査では証明していない。
