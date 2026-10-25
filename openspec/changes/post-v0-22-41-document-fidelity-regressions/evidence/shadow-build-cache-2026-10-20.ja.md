# 比較検証のビルドキャッシュと結果を分離する

## 目的と境界

製品shadowは約19GiBの生成物を残して容量安全中断した。同じ依存関係を新しいtargetへ再コンパイルして容量を二重消費しないため、診断専用`--build-cache-target`を追加する。通常test・coverage・CI・品質閾値は変更しない。

利用するのはCargoがfingerprintで再コンパイルを判断する通常のビルドキャッシュだけである。古いNextest結果や成功receiptを読み込む経路はない。元runのsource/lock/toolと同一という追加要件は設けず、現在のsource/lock/tool・実行結果を毎回新しく記録する。`execution_equivalent`、`reuse_authorized`、`report_checked`はfalseのまま。

## 安全契約

- 対象は物理repo直下の`tmp/nextest-shadow/<一つのrun名>/coverage-target`に限定する。移動・削除や任意targetの選択は行わない。
- canonical絶対path、祖先・全子孫のsymlink拒否、正規Cargo `CACHEDIR.TAG`、regular file、深さ16・entry数65536の走査上限を確認する。tagは4KiB+1の有界read、nofollowで開き前後identityを照合する。
- 既存profile全てへnonblocking lockを取得し、直ちに解放する。busy・未知profile・未確認filesystemは拒否する。これは事前確認であり全runの排他保証ではない。Cargo自身のlockと既存担当間の実行枠調整を維持する。
- 出力directory、Nextest state、全4工程の実行、workerのbuildコマンドとreadonly snapshotは毎回新規に作る。旧PASSを使って工程を省略しない。
- run開始時にUUIDを生成し、各工程のprofile basenameへ含める。選択target・UUID・工程ごとの実basenameをreceiptへ保存する。古いprofrawを採用したcoverage reportの合格は主張しない。

## 検証の回収地点

実partial cacheは主査の事前validatorで受理された。製品コンパイル・移動・削除は行っていない。最終の正規`just test-nextest-shadow-contract`は42件PASS・2.741秒・session66447/actual0。rawは`tmp/oct20-shadow-cache-contract-final.log`。invalid cacheでtool child自体を開始せず、出力も作らない実CLI回帰を含む。全41件の中間成功とは区別する。

依存0の実Rust fixture`tmp/nextest-shadow-probe/2026-10-20-cache-retry`で、seed76805/actual0→fresh retry15128/actual0を回収し、主査も両`shadow.json`とrawを実読した。両runでworker buildと全4工程がexit0、4store、source clean/stable、falseの各policyを確認した。同じ物理targetを使ってもworker snapshot pathとrun UUIDは別である。追加した`old-seed.profraw`は保持され、record収集対象ではなかった。これはcoverage report検証ではない。

- seed UUID: `8e0b8f335d714b5dbaed9bd4c6c9344a`
- retry UUID: `eeff1b5cca0a4c32adc0ec67c0ae4df7`
- seed raw: `tmp/oct20-shadow-cache-seed.log`
- retry raw: `tmp/oct20-shadow-cache-retry.log`
- 検証記録: `tmp/oct20-shadow-cache-verify.log`

tiny fixtureでは二回目のcompileを含む工程時間が短縮したが、製品の時間短縮・全体同等性とは主張しない。独立最終レビューで必須P0/P1/P2は0件。実partial cacheのreadonly計数は31,020 entries・最大dir深さ8・symlink0であり、上限内であることも確認された。通常commitは別に回収する。

製品shadowの再実行はKDVの合意済み重検査枠の後に行う。この文書やtiny fixtureの成功を、製品全工程・実行同等性・速度改善・全OS・公開の証明へ置換しない。
