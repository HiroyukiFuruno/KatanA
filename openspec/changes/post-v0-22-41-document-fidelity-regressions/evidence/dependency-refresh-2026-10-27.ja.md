# 本人による依存更新の取り込み

2026-10-27、本人が実行した `just update` の差分を今回の検証対象へ取り込む。
依存更新の確認をリリース直前だけに限定せず、長期作業中も公開された修正を評価する。
KDV/KRRの追加修正を待たず先行公開する判断は維持する。

## 変更

| 依存 | 旧lock | 新lock |
| --- | --- | --- |
| comrak | 0.57.0 | 0.59.0 |
| notify | 8.2.0 | 8.2.1 |
| resvg / usvg | 0.48.1 | 0.49.0 |
| zip（直接依存） | 9.0.5 | 9.1.2 |

rootと撮影ツールの両lockを更新した。
推移依存にはbrotli-decompressor、ctutils、fearless_simd、lzma-rust2、ordered-float、
roxmltree、rustix、typed-pathなどの更新も含まれる。
notifyのmacOSバックエンドではfsevent-sysからobjc2-core-servicesへ変化しているため、
既存のファイル監視回帰も正式検査で維持する。

KDVはregistryの `=0.5.12`、KRRは `=0.4.23` のままであり、両lockも同じ版を解決する。
未公開siblingのpath/git overrideを追加していない。
既存のworkspace内pathとリポジトリ内vendorは別の境界として維持する。

## 検証と残件

依存更新後、workspaceの対象テスト群をコンパイルし、通常側の診断ロジック3件が成功した。
これは全テスト成功、全描画の同一性、配布起動や公開の証明ではない。
SVG、Markdown、ZIP入力、ファイル監視を含む既存の必須検査を通常push・新CIで実行する。
供給網検査の鮮度と既存の限定例外の検査も維持する。

今回の直接ZIP更新で、KDV配下の旧ZIPが消えたとは扱わない。
ライブラリの影響版の存在、実際に利用するAPI、発生条件を分けた既存評価は継続する。
採取成功テストの診断専用化は別の関心事としてコミットする。
