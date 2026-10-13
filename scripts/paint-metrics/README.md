# 実描画の文字座標計測

`just test-paint-metrics` は実 egui フレームの文字メッシュを使い、座標の平行移動、複数行、空白、回転の未対応判定を検証する。KatanA全体の受入試験の代替ではない。

KatanAでの採取は既存 screenshot runner に `scripts/screenshot/examples/sample-typography.json` を渡す。`record_typography` は指定した文字列が描画されていなければ失敗し、JSONに実フレームの全文字、行、フォント指定、source anchorを保存する。

- 座標は画面原点のlogical point。物理pixelへの倍率は `pixels_per_point`。
- `origin` と `galley_rect` はTextShapeの配置と文字レイアウト領域であり、親widgetのouter rectではない。
- 各glyphの `baseline` は画面上の基準点。`baseline_offset_from_row_top` は行上端から基準点までの距離。
- `glyph_band` は文字メッシュ頂点の境界。クリップ前で、テクスチャ余白を含み得るため、ラスタ画像の非透明ink領域とは区別する。
- `heading_hit` は既存の合成された見出しアンカー。実ブロック外形として使わない。
- 本文段落には既存source anchorがない場合がある。描画文字の一致をsource lineの実測と混同しない。
- 回転文字は `unsupported_rotation` と記録し、誤った未回転座標を返さない。

正式な依存採用と実ホストの実測が終わるまで、計測機能の単体テスト成功を表示互換性やリリース完了の証拠にしない。
