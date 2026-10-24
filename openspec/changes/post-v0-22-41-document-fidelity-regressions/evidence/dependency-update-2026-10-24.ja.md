# 本人の依存更新と互換性検証（2026-10-24）

## 保持した更新と見送った更新

本人の `just update` による4つのCargo.toml変更を精査し、comrak0.57、ureq3.5、emojis0.9.1、pulldown-cmark0.13.5、open5.4.5、ehttp0.7.2、unicode-normalization0.1.26、arboard3.7、uuid1.29、skrifa0.49、write-fonts0.55の更新を保持した。rootと撮影ツールの両Cargo.lockを通常の公開registry依存から再解決した。

- parking_lot0.13はparking_lot_core0.10経由でlibc0.2.191以上を要求するが、公開KDV0.5.12はlibc0.2.190へ固定している。KDV/KRRの次版を待たない本人方針に従い、platform/UIの両方を最新互換版 `=0.12.5` へ固定した。通常のupgradeで同じ非互換更新を繰り返さず、上流更新時に再評価する。
- egui_commonmark0.26はegui0.37を追加し、現在のegui0.36.2と既存vendor0.25の描画修正を置き換えてしまう。実際の更新ログでunused patch警告を確認し、既存修正を維持する `=0.25.0` へ固定した。既存vendor以外の未公開sibling path/git overrideは追加しない。
- KDV `=0.5.12` / KRR `=0.4.23` の公開registry固定は維持した。旧ZIP2系は2.6.1へ更新されたが、9未満の既知advisoryを解消したとは扱わない。

## 実行結果

- `just update-safe`: 最終実行session88723、actual0。egui0.37が除去され、既存commonmark/backend0.25のvendor修正が有効になった。
- 撮影ツールlockの通常更新: session94287、actual0。
- 両lockを解析し、libc0.2.190、parking_lot0.12.5、egui0.36.2、KDV0.5.12、KRR0.4.23が一致することを確認した。
- 正規 `just type-check`: session93555、actual0。`tmp/oct24-dependency-type-check-final.log`。先行実行20814は同時進行していた更新ZIPモジュール分割のpath欠落でactual101となり、修復後の全体成功と区別して保持した。依存APIの非互換性として隠蔽していない。

これは依存解決と全体コンパイルの証拠であり、新HEADの全テスト・coverage・全OS・配布受入・公開の完了ではない。これらは通常pushとReady後の正式工程で確認する。
