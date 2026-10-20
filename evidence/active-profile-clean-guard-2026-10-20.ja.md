# 既存Cargo profileを削除から保護する限定修復

## 不備と再現

従来の `just clean` は、先行するsweepがbusyのため安全にskipしても、その後に無条件で `cargo clean` を実行した。Cargo clean自身は実行中buildのprofile lockを待たず削除できる。

実Cargoの小fixtureをbuild scriptのbarrierで停止して旧コマンドを実行したところ、cleanは0で終了し、targetとdebug/.fingerprintを削除した。barrier解放後のbuildは101で失敗した。rawは `tmp/cargo-clean-active-red-2026-10-20.log`。実repositoryや他repoのtargetはこの再現では削除していない。

## 修復範囲

- 既存のtarget検出・local filesystem判定・profile列挙・flockを再利用する。
- 既知profileのlockを取得できない場合、clean本体を実行せず75で拒否する。
- 不明な構造、symlink、root/home/manifest rootとそのancestorを拒否する。
- 正規CARGOの環境代入・wrapperを保持し、検証済み絶対targetへcleanを固定する。
- 子プロセスの出力・終了コードを保持し、finallyでlockを解放する。
- fcntlを利用できない環境では削除せず、安全拒否として扱う。

## 検証と限界

主査が正規 `just test-cargo-sweep-contract` を実行し、初回11件/32.759秒/session14852 actual0を回収した。実active build中の拒否、fingerprint保持、barrier解放後build0、idle clean0、危険path拒否、実CARGO wrapperのmetadataを含む。

レビューで見つかった3OS CIの旧suite限定配線を修復し、正規CIも同じglobで新suiteを実行するよう変更した。実CARGO wrapperによるidle cleanと、実cargo clean後に23を返すwrapperの終了コード伝播も追加した。非対応OSでは75とartifact保持を期待し、Unixの削除成功を代用しない。

- 最終正規13件/4.132秒/session36394 actual0。raw: `tmp/oct20-active-clean-final13-platform.log`。
- CI resource契約10件/0.008秒/actual0。raw: `tmp/oct20-active-clean-ci-contract.log`。
- 固定actionlint1.7.12/session69117 actual0。raw: `tmp/oct20-active-clean-actionlint.log`。
- 最終3OSの実CI結果は未回収であり、上記は現macOSでの結果と配線検査に限定する。Draft CIは起動していない。

これは既存profileのlock競合に対する改善であり、列挙後の新profile生成、target差替え、guardを通さない直接cargo cleanまで全面排他する設計ではない。全Cargo入口を共有排他へ参加させていないため、同時に新profileを作らない運用調整は引き続き必要。安全なcleanup全体の完成とは扱わない。

今回の製品core検査9365はV8 dep-info書込み先消失errno2でactual101となったが、その削除actorと原因は未特定。この再現経路が当該中断の原因だったとは断定しない。製品のZIP互換性、core全test、Windows実行、全release gate、公開は別の未完条件。
