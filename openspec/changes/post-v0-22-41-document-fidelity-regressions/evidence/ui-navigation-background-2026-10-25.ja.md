# Markdown分割背景と文書ページ移動

## 実装

- #355: Markdownの左右・上下分割が通る共通入口で、Explorerと同じ既存テーマのwindow背景を塗る。他形式とコード・表の装飾色は変更しない。
- #356: PDF/PPTXの文書面をクリックしてフォーカスした場合に、修飾キーなしの左右キーを既存のPrevious/Nextコマンドへ渡す。他形式・他widgetのフォーカス・先頭末尾境界では送信しない。

## 対象検証とレビュー

- 背景の描画色・Markdown拡張子・他形式を確認する対象テスト1件成功。
- 実eguiキーイベントとフォーカス、形式、修飾キー、ページ境界を確認する対象テスト2件成功。初回の0件filterは成功証拠へ使わず、修正後の2件実行を採用。
- 独立ソースレビューでP0/P1なし。レビュー対象3ファイルのSHAと現在のファイルを主査が照合し一致。
- 通常commit時ASTがliteral色とrenderファイル200行超を検出し、21成功/2失敗で停止。テーマ由来の色とページ移動のprivate責務分離で修復し、背景1・キー2・AST30・影響Clippy・fmt/diffをすべて実exit0で回収。レポート`tmp/oct25-ui-validation-repair.json`。
- 原記録: `tmp/oct25-ui-independent-review.md`。最終修復と数値exitは`tmp/oct25-ui-validation-repair.json`、最終5file独立レビューは`tmp/oct25-ui-final-independent-review.md`へ保存。

## 公開条件との関係

本人の最新指示により、自動検査・通常レビュー・必須CI・公開を担当範囲とする。本人による実機確認は公開後の独立した確認であり、Issue Closeや公開を止める条件にしない。実機未確認、既知のメモリ未達やOffice多タブ切替の未実測を解消済みとは扱わない。背景色は署名G `b6485f51`、ページ移動は署名G `f51f371e`へ通常commitし、各hookの実exit0を回収済み。新HEADのpush・レビュー・必須CI・全配布物公開は未完。
