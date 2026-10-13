## Why

v0.22.26 では `katana-document-viewer` 経由の図形描画処理は入ったが、KatanA 側の直接依存、backend version、OpenSpec、README に `katana-diagram-renderer`（KDR）名義が残った。KDR は `katana-render-runtime`（KRR）の互換 wrapper であり、KatanA の現行依存先として扱ってはならない。

## What Changes

- KatanA の workspace dependency を `katana-diagram-renderer` から `katana-render-runtime = "0.3.3"` へ切り替える
- 図形 preview / export の backend id、cache version、runtime asset 解決を KDV + KRR 境界へ更新する
- README と OpenSpec の現行仕様を KDV + KRR に同期し、KDR を現在の依存先として記載しない
- `katana-document-viewer` v0.1.1 が crates.io に公開された後、KatanA v0.22.27 の release dependency として取り込む

## Impact

- 追加: `katana-render-runtime` v0.3.3 を crates.io dependency として参照する
- 削除: KatanA から `katana-diagram-renderer` への直接依存を外す
- 変更: 図形描画の識別情報は `renderer=katana-render-runtime:<version>` を使う
- ブロッカー: `katana-document-viewer` v0.1.1 の crates.io publish が完了するまで、KatanA v0.22.27 の release dependency は確定しない
