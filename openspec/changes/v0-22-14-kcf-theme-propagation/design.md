## Context

画面上では、KatanA の UI が light テーマでも、Markdown プレビュー内の Mermaid / Draw.io 図形が濃い背景と白文字寄りの dark 配色で表示される。

調査した経路は次の通り。

- KatanA preview は `crates/katana-ui/src/preview_pane/renderer_dispatch.rs` で `DiagramThemeSnapshot` を作り、`DiagramBackendInput` へ渡している。
- KatanA core は `crates/katana-core/src/markdown/diagram_backend/katana_backend.rs` の `kcf_input()` で、kcf の `RenderInput` に `RenderPolicy.background` / `cache_profile` / `RenderContext.theme_fingerprint` を渡している。
- kcf 側は `crates/katana-canvas-forge/src/renderer/backends.rs` で `RenderInput` を受け取るが、実描画では `input.source` だけを `DiagramBlock` へ詰め、テーマ情報を描画層へ渡していない。
- kcf Mermaid 描画は `crates/katana-canvas-forge/src/markdown/mermaid_renderer/render.rs` で `DiagramColorPreset::current()` を直接参照しており、kcf 側の `DARK_MODE` 初期値は `true` である。

このため、根本原因は「KatanA が light 情報を渡していない」ではなく、「kcf 側の renderer が `RenderInput` 由来のテーマを実描画に使っていない」ことにある。kcf 側 issue は [HiroyukiFuruno/katana-canvas-forge#4](https://github.com/HiroyukiFuruno/katana-canvas-forge/issues/4) として起票済み。

## Goals / Non-Goals

**Goals:**

- kcf 修正版を取り込み、KatanA の current theme が kcf backed Mermaid / Draw.io 描画へ反映されるようにする。
- KatanA adapter が kcf の新しいテーマ入力契約を満たしていることを、unit test と preview/export 経路の回帰テストで確認する。
- cache key と kcf の `cache_fingerprint` が、実描画で使われたテーマ差分に追従することを確認する。
- light テーマで Mermaid / Draw.io が dark 配色へ戻らない screenshot 証跡を用意する。
- kcf の crate version / runtime / renderer profile を手書き文字列で固定せず、依存版や kcf 出力から取得する。

**Non-Goals:**

- kcf の内部 renderer を KatanA 側へ戻すこと。
- KatanA 側で kcf の欠落を一時的に補う独自 Mermaid / Draw.io 実装を再作成すること。
- 新しい図形言語や新しい export 形式を追加すること。
- テーマ設定 UI の見た目を変更すること。

## Decisions

1. kcf 側修正を前提にし、KatanA 側でグローバル状態の同期に頼る回避策を入れない。

   `katana_canvas_forge::markdown::color_preset::DiagramColorPreset::set_dark_mode(false)` のように kcf 内部状態を KatanA から直接操作すれば、見た目だけは一時的に揃えられる。しかし、それでは `RenderInput` を持つ公開APIの意味が崩れ、複数 consumer や並列描画で再発しやすい。KatanA は current theme を DTO（データ受け渡し用の型）へ詰め、kcf はその DTO を実描画に使う境界に戻す。

2. KatanA の `DiagramThemeSnapshot` を kcf の新しい typed theme 入力へ変換する専用 adapter を置く。

   kcf 側で `RenderPolicy` が拡張されるか、別の theme DTO が追加されるかは issue #4 の実装に従う。KatanA 側では `kcf_input()` に変換を直書きし続けず、テーマ変換責務を小さく分離して、preview と export の両方から同じ変換を使える形にする。

3. kcf version / runtime / renderer profile は hardcode しない。

   現状の `KCF_MERMAID_BACKEND_VERSION` / `KCF_DRAWIO_BACKEND_VERSION` は `crate=katana-canvas-forge:0.1.0` を手書きしており、実際の workspace dependency `0.1.1` とずれている。cache invalidation（キャッシュ無効化）は重要なので、kcf の `RenderOutput.runtime` / `RenderOutput.profile` または crate metadata から取得し、手書きの更新漏れを起こさない。

4. 回帰テストは「light を渡した結果」まで見る。

   cache key が変わるだけでは、kcf が実際に light 配色で描いた証明にならない。KatanA 側では、adapter が kcf へ渡す入力の検査、kcf 修正版で返る `cache_fingerprint` の差分、light テーマ screenshot の3層で確認する。

## Risks / Trade-offs

- Risk: kcf 修正版の公開APIが issue #4 の実装で変わる  
  → KatanA 側の変換は専用 adapter に閉じ込め、API差分の影響範囲を `diagram_backend` に限定する。

- Risk: kcf dependency 更新で Mermaid / Draw.io の既存レイアウトが変わる  
  → KatanA 側では light/dark の代表 fixture に加え、既存の Mermaid / Draw.io integration test を通す。詳細な描画採点は kcf 側の reference compare に任せる。

- Risk: export は preview と別 thread で実行されるため、テーマ取得時点がずれる  
  → export 開始時に current theme snapshot を明示的に捕まえ、thread 内ではグローバル状態を読まず snapshot を渡す。

- Risk: 既存の永続 diagram cache が dark 配色のまま残る  
  → テーマ fingerprint と kcf runtime/profile が cache key に含まれることを確認し、必要ならこの変更で cache version を進める。
