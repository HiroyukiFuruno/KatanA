## Why

v0.22.43公開後も、Office/HTML読込中の画像選択が停止し、共通ローディングの水平位置とPPTX/PDFの左右キー操作が期待どおりでないと報告された。単独画像の高速測定や共通部品の存在だけでは、混在操作の品質を証明できなかった。

## What Changes

- 全報告をIssue #388/B01〜B11と新規報告へ照合し、公開済みの限定修正と残る不具合を区別する。
- 読込中Office/HTMLと画像表示の実経路、待ち行列、取消、キャッシュ、終了を調査し、KatanA所有の阻害要因を修正する。
- 共通LoadingIndicatorと全callerの配置を上下左右中央へ統一し、実egui矩形を自動検査する。
- 文書選択後のPPTX/PDFページ送りと既存キーバインド/入力フォーカスの競合を修正する。
- 原本と既存回帰を使い、未再現や上流原因を解消済みと偽らず記録する。

## Capabilities

### New Capabilities

なし。

### Modified Capabilities

- `multi-format-document-preview`: 非同期混在操作、共通読込配置、文書ページ送りの入力所有を明示する。

## Impact

katana-uiの文書プレビュー、画像読込、キーボード入力と自動回帰。公開APIの追加・sibling実装・新規worktree・未公開依存への切替は行わない。公開済みv0.22.43は上書きせず、release/v0.22.44で通常履歴とレビューを進める。
