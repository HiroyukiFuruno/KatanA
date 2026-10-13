## Context

KatanA はホスト UI、KDV は PDF/Office の解析・状態、KRR は HTML ブラウザー実行を所有する。添付ファイルから判明した不具合はこの境界をまたぐため、上流を先に公開して registry 経由で下流へ取り込む必要がある。

## Goals / Non-Goals

**Goals:**

- KatanA はエクスプローラー選択、入力面、薄い操作投影だけを担当する。
- KDV 0.5.5 のシート名と PDF 目次を KatanA の UI へ投影する。
- `office2pdf-katana 0.6.10` → KDV 0.5.5 → KatanA v0.22.41 の順で公開する。
- 添付された HTML と Office 6件を実ホスト経路で回帰検証する。

**Non-Goals:**

- KatanA 内に PDF/Office/HTML の別レンダラーを実装しない。
- Office のアクティブコンテンツを実行しない。
- path/git 依存や品質ゲートの緩和をリリースへ持ち込まない。

## Decisions

- HTML は KRR のブラウザー面を入力対象に保つ。静的描画へのフォールバックは操作要件を満たさないため採用しない。
- XLSX は KDV の `item_labels` を下部タブへ表示し、`JumpTo` だけを返す。シート解析を KatanA で重複させない。
- PDF は KDV の `PdfOutlineItem` を階層表示し、解析済み `page_index` へ `JumpTo` する。PDF オブジェクト解析は KDV に限定する。
- Office 目次の利用可否とアクション拒否を両方で守り、状態だけが開く経路を残さない。
- sibling 修正版は crates.io 公開後に exact registry 依存として固定する。未公開版を KatanA の最終 lockfile に残さない。

## Risks / Trade-offs

- [PDF に目次が埋め込まれていない] → 空の目次として扱い、推測生成は行わない。
- [シート名が欠落・空文字] → KDV 契約を優先しつつ、KatanA は番号ラベルへ安全にフォールバックする。
- [fork の upstream 追従が遅れる] → 一時的な bridge とし、上流修正版公開後に公式 crate へ戻せる依存境界を維持する。
- [大容量 XLSX のメモリ消費] → KDV の展開量・アーカイブ量上限とストリーミング解析で制限する。

## Migration Plan

1. `office2pdf-katana 0.6.10` を検証・公開する。
2. KDV 0.5.5 を公開 registry fork と KRR 0.4.17 で検証・公開する。
3. KatanA v0.22.41 を公開 KDV/KRR に更新し、実ファイル受入と全リリースゲートを実行する。
4. PR をマージして GitHub Release と配布物を検証する。問題時は未公開の下流リリースを進めず、直前の公開版を維持する。

## Open Questions

なし。
