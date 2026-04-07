## Technical Architecture

v0.16.8 のリッチブロック処理安定化実装に基づき、以下の点を微調整します。

1. **ホバーマッピングの境界修正**
   プレビュー内で取得される `egui::Rect` と `pulldown-cmark` が提供する `Range<usize>` 間のマッピングにおいて、余分なマージン等による境界ジャッジのズレを吸収するため、ホバー判定に使用する閾値や計算ロジック（`egui_commonmark` 内のブロックアンカー収集など）を補正します。

2. **スクロール同期 (ドリフト解消) の再調整**
   `ScrollMapper` に登録されたリッチブロックの始点・終点ポイントでの単調性違反（Non-monotonic points）の発生原因となる `0V` の要素や意図しない高さを微調整し、`DEGENERATE_EPSILON` の見直しや補間ロジックの改善を行います。

## Module Specifications

### UI レイヤーの微調整

1. **PreviewPane レンダリング (プレビューのヒットテスト)**
   - `crates/katana-ui/src/preview_pane/ui.rs` にて、マウス位置と `block_anchors` を突き合わせる際のオフセット計算および Rect の `intersects` / `contains` ロジックを精査します。

2. **ScrollSync マッピング**
   - `crates/katana-ui/src/state/scroll_sync.rs` にて、`add_map_point` に渡る座標が実際のUI上の見え方と完全に一致するよう、リッチブロック描画側のY座標の供給タイミングとマージン計算を修正します。
