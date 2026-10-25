# 製品shadow再実行（2026-10-20）

## 実行と固定入力

- 対象HEAD: `01c54d4bd8c0fbc36139bd89b33bbd77f1329ff2`。開始・終了ともclean、同HEAD・同tree・両lock SHA一致。
- session `72117`、終了 `2026-10-20T14:15:16Z`、実終了code `0`。
- 出力: `tmp/nextest-shadow/product-01c54d4b-20261020/shadow.json`。
- 生ログ: `tmp/oct20-product-01c-profile-shadow.log`、確認ログ: `tmp/oct20-product-01c-profile-shadow-verify.log`。
- 前回の`product-85c01a7d-20261020/coverage-target`のコンパイルcacheだけを明示利用。全テスト・worker build・snapshot・4個のNextest storeは新規実行した。
- run namespace: `9007f6ec4b3241d48446095574668642`。各phaseの`%4m` profileはrun UUID・phase名で分離。
- worker snapshot SHA256: `661401c593c9191ce8a79d6689aa528e329f59970338b2ddd66372ac894dad39`。

## 実結果

| 工程 | 実終了 | テスト結果 | build込み秒 |
| --- | --- | --- | --- |
| worker build | 0 | snapshot生成 | 6.058 |
| workspace lib/bin | 0 | 1770 PASS、既存2 skip | 82.607 |
| workspace integration | 0 | 228 PASS、既存3 skip | 560.877 |
| UI parallel | 0 | 144 PASS、既存2 skip | 475.924 |
| UI serial | 0 | 18 PASS、skipなし | 14.672 |

計2160 PASS、既存7 skip。`producer_completed=true`、`source_stable=true`を主査がreceiptと生ログへ照合した。
図の複合書き出しケースは341.578秒で成功。linkerの`__eh_frame`警告は生ログに残し、警告なしとは扱わない。
終了時空き容量は約3.7GiB。KDV担当による自所有idle cacheの保護回収を途中で受け、他repoやglobal cleanupを本担当では行っていない。

## 未完了と境界

前回の容量中断130を成功へ書き換えず、新規runの成功として回復を記録する。
`execution_equivalent=false`、`reuse_authorized=false`、`report_checked=false`を維持する。
通常testとの同等性、coverage閾値、実重複削減・速度短縮、全OS、canonical採点、packaged受入、公開完了を証明したものではない。既存normal/coverage/Ready CIの品質条件は変更していない。
