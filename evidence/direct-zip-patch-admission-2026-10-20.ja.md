# 直接ZIP patch更新の検証途中記録

## 対象

公式registryのZIP9.0.5とRust1.88の最低要件・公式CHANGELOGを確認し、rootとscreenshotの両lockをZIP9.0.2→9.0.5、lzma-rust2 0.21.0→0.22.0だけ更新した。manifest、固定KDV0.5.12/KRR0.4.23、上流が使用する旧ZIP3版は変更していない。

root/screenshotの各dry-runと実Cargo updateはactual0。新ZIP checksumは `6e04976da629d3b04204ba8105b396a193af8ad2583b2c79bfb3ab8af580a11e`、lzma-rust2は `7277c9742229dc25243236d578b6471a798c554512c6f6a04365ec2e87c2a3bb`。

## 回帰と未完条件

実consumer `UpdateDownloadOps::extract_update` に、実ZipWriterで生成したDeflated/Xzのentry method再読、nested bytes、完了progress、Unix0755保持を確認する2件を追加した。製品コード・公開APIを変更せず、Unix権限検査はUnixだけで実行する。formatterと差分検査は成功した。

最初の正規one-job `just test-core`（session9365）は、V8のdep-info書込み先target/.fingerprintが消失したためerrno2・actual101で中断した。rawは `tmp/oct20-zip905-core-test.log` とRTK tee `1792484547_cargo_test.log`。両lock/consumerの入力SHAは開始終了一致。削除actorは未特定であり、ZIP互換性の失敗とは判定しない。再実行は他担当の重いCargo検査と順序を調整する。

## 実consumerの限定検証

重い製品batchと競合しない小temp crate `tmp/zip-consumer-admission` を作り、製品のdownload.rs/types.rs/proxy_env_test.rsと新しいarchive_tests.rsをpathで直接読み込んだ。処理のコピーや代用品は使っていない。root lockをコピーし、7つの直接依存を現rootと同じexact版・ureq json/default ZIP featuresで指定した。KDV/KRR/V8/GUIはこの限定graphに含めない。

- 新2件: session9715 actual0、初回build1分11秒、実行0.01秒。Deflated/Xz・Unix0755・bytes・progressが成功。raw: `tmp/oct20-zip905-consumer-isolated.log`。
- 既存3件: actual0、実行0.00秒。実HTTP download・Stored展開・nested/path traversal拒否が成功。raw: `tmp/oct20-zip905-consumer-existing.log`。
- build中に観測した製品両lock/実consumer/新testとtemp manifest/lockのSHAは終了時にも全一致。raw: `tmp/oct20-zip905-consumer-observed-inputs.sha256`。build開始前の完全なclean HEAD証明とは区別する。
- temp graphに読み込んだ未使用の型などのwarning8件は保持し、製品strict lint成功とは扱わない。生成targetは約361MiB、one jobで実行した。

この成功は現macOSの更新consumerに限定する。製品core全体・screenshot全体・全依存graph・3OS・normal hook/CI・packaged・公開の成功へ流用しない。通常commitの結果は次節に分離し、正規製品batchは未完条件として維持する。

## 通常hookと正式履歴への統合

初回通常commitはAST22成功/1失敗でactual1。新testのWHY行コメント形式と0755 literalを指摘されたため、既存規約どおりblock WHYとnamed constantへ限定修復した。assertion・Unix0755の値・閾値は変更していない。raw: `tmp/oct20-zip905-normal-commit.log`、RTK tee `1792487883_cargo_test.log`。

修復後の通常commit hookはactual0、sourceを署名Gへ統合した。作業者の英語messageを規約どおり日本語へ訂正した未公開tipは `cc5a6600`。訂正前後tree/indexは `565f3600d27f0ba3c109b2da773461fd6e9523a8` で不変、amendも通常hook actual0。署名・hookを迂回せず、他の履歴やremoteは変更していない。raw: `tmp/oct20-zip905-normal-commit-retry.log`、`tmp/oct20-zip905-normal-commit-amend.log`。

上記5件の実consumer成功はこの形式・定数修復より前の実測であり、最新HEADの全製品テスト成功と読み替えない。修復後の形式検査は通常hookで成功した。単独ASTを重複実行しておらず、成功件数の新たな推定値は記載しない。最新sourceの製品core全testは次節で回収した。全3OSは元の正式ゲートで回収する未完条件として保持する。

最終read-onlyレビューではP0/P1/必須P2なし。検証終了後、検証専用targetだけを新clean guardで削除しactual0/366.8MiBを回収した。source・lock・入力SHAは削除後も全一致で、fixtureとrawログを保持している。cacheは再生成できる。raw: `tmp/oct20-zip905-consumer-cache-clean.log`。実repositoryのtargetや他担当のcacheは削除していない。

## 現graphの正規core回収

KUCの共有検査batch実終了後、`CARGO_BUILD_JOBS=1 just test-core` を一回だけ実行しactual exit0を回収した。RTK集計は19 suites・407 passed・既存2 ignored・272.62秒。開始10:47:17Zと終了10:57:16ZのHEADは `e0e90d23f5b15ebc57e55dcecd4ec7213573a750`、source clean、両lock SHAも一致した。旧9365の失敗はこの成功とは別に保持する。

- root lock: `2402275790e13ae3dbf811470a7a76fa8f5385164f1ee591af7597e0d4f9a395`
- screenshot lock: `aea404de57de7e2ad9955adb33c3e999d0be8d4d0ae223a5e6b489ab73b32377`
- raw集計: `tmp/oct20-e0-current-core.log`
- identity: `tmp/oct20-e0-current-core-start.log` / `tmp/oct20-e0-current-core-end.log`

形式修復後の製品core検証であり、全workspace・Windows/Linux・coverage・packaged受入・供給網・公開の成功へ広げない。実終了後にKUC/KDV担当へ枠を返し、追加Cargo検査を起動していない。

## 供給網との分離

変更後root供給網（session35203）とscreenshotの独立deny（session59680）は、既知の上流旧ZIP3版に対するRustSec0367/0368/0369でactual1だった。bans/licenses/sourcesは成功したが、全供給網成功ではない。rootの正規ターゲットは最初のdeny失敗で停止するため、後続チェックが走ったとは扱わない。

raw: `tmp/oct20-zip905-root-supply-chain.log`、`tmp/oct20-zip905-screenshot-supply-chain.log`。新しいZIP/lzmaの警告追加は確認されていないが、上流安全性修正版待ちをこの直接更新で解消したとは主張しない。検査失敗を迂回してpush・Ready・公開しない。
