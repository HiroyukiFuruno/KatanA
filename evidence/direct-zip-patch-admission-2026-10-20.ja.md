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

この成功は現macOSの更新consumerに限定する。製品core全体・screenshot全体・全依存graph・3OS・normal hook/CI・packaged・公開の成功へ流用しない。通常commitと正規製品batchは未完条件として維持する。

最終read-onlyレビューではP0/P1/必須P2なし。検証終了後、検証専用targetだけを新clean guardで削除しactual0/366.8MiBを回収した。source・lock・入力SHAは削除後も全一致で、fixtureとrawログを保持している。cacheは再生成できる。raw: `tmp/oct20-zip905-consumer-cache-clean.log`。実repositoryのtargetや他担当のcacheは削除していない。

## 供給網との分離

変更後root供給網（session35203）とscreenshotの独立deny（session59680）は、既知の上流旧ZIP3版に対するRustSec0367/0368/0369でactual1だった。bans/licenses/sourcesは成功したが、全供給網成功ではない。rootの正規ターゲットは最初のdeny失敗で停止するため、後続チェックが走ったとは扱わない。

raw: `tmp/oct20-zip905-root-supply-chain.log`、`tmp/oct20-zip905-screenshot-supply-chain.log`。新しいZIP/lzmaの警告追加は確認されていないが、上流安全性修正版待ちをこの直接更新で解消したとは主張しない。検査失敗を迂回してpush・Ready・公開しない。
