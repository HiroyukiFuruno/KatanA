# 保存済みNextest記録の再生検証

## 対象と結果

2026年10月21日、Issue #343 の既存shadow記録を固定Nextest
`0.9.148` の公式 `replay --exit-code` で検証した。製品テストの再実行、
ビルド、CI起動、本人アプリの操作はしていない。

対象は `tmp/nextest-shadow/product-01c54d4b-20261020/` にある過去の実行記録である。
現HEAD `6097058c6fc52c6ad0610e961906ebcc8b2e8cd5` の製品検証ではない。

| 工程 | 記録run UUID | 公式再生exit | 元記録の成功 / skip |
| --- | --- | --- | --- |
| workspace-lib-bin | 07326a0a-7c7b-4f29-b319-a7a8bf021551 | 0 | 1770 / 2 |
| workspace-integration | ce3ededa-0abb-4678-8d34-a08718850b00 | 0 | 228 / 3 |
| ui-parallel | ca6096b9-44f3-4512-9686-d665d2651d2c | 0 | 144 / 2 |
| ui-serial | bd11568b-ee82-4d4f-b3c7-f0c60392f35c | 0 | 18 / 0 |

各工程の `phase.json` から独立した `NEXTEST_STATE_DIR` とrun UUIDを指定した。
`store.zip` のSHA-256を既存receiptと比較し、再生前後でも一致した。
全工程の元child exitと公式再生exitはいずれも0であった。

公式の[再生機能](https://nexte.st/docs/features/record-replay-rerun/replay/)では、
通常の再生成功だけでは元テストの終了状態を表さないため、`--exit-code` を指定した。
表示された時間は元記録の所要時間であり、今回の再生時間や高速化量ではない。

## 一次証跡

- 実行手順: `tmp/verify-oct21-retained-replays.py`
- 4工程のargv、UTC開始終了、元exit、再生exit、receipt/store hash:
  `tmp/oct21-retained-replays/results.json`
- 各工程の公式再生出力: 同ディレクトリの工程名別 `.log` 4件
- 主査実行結果ID: `e475dd`、全体exit 0、約0.55秒

## 判定と残作業

完了したのは、保存済み4工程を公式readerで再生でき、元の正常終了状態を
確認できたという受入である。自前のZIP内部parserや非対応の
`replay --message-format json` は採用していない。

この結果から、通常libtestとの実行集合同等性、instrumented / non-instrumentedの
挙動同等性、検査省略権限、trusted reuse、Windows cold/warm、現在の全OS品質、
packaged受入、公開完了は導けない。`execution_equivalent=false` と
`reuse_authorized=false` を維持し、既存の正式検査を削減していない。

Windows cache履歴は既存10月13日証跡と再照合し、旧無効化に技術的理由の
記録がないこと、registry-only再利用とrelease target cacheが別経路であることを
確認した。これは新規実装ではない。残るWindows Ready実測を完了扱いせず、
根拠のないtarget cache有効化やDraft CI起動は行わない。
