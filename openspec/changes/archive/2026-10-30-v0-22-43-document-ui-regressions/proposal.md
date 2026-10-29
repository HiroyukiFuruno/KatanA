## Why

v0.22.42で、約100KBの画像表示に約30秒かかる報告、タブグループ操作不能、HTML・Office終了時のフリーズ、形式に合わない右メニューが報告された。見た目だけの個別修正で終わらせず、実経路の回帰と共有設定で次のパッチ版へ修正を届ける。

## What Changes

- 画像の要求から初回表示・変更なし反復までを調べ、確認したKatanA側原因を修正する。待機表示の修正を速度改善の証明にしない。
- 全右メニュー・モード・ショートカット・palette・直接実行の可否を、文書形式と現在の描画状態から得る共通設定へ統一する。共通Loading部品へ文言を渡す。
- タブグループ名入力の確定・取消・外側クリック後の操作を回復する。
- HTML/DOCX/XLSX/PPTXのcloseを保存・preview解放・worker・メモリ返却へ分けて計測し、KatanA側の同期停止を修正する。
- 画像制御を通常・全画面とも画像下部の独立領域へ移し、小さい画像の全画面ボタンとの重なりを防ぐ。
- 配布同期の失敗検出とHomebrew生成DSL、Docker資源の所有・証跡保存・対象限定回収を整える。

## Capabilities

### New Capabilities

- `docker-resource-lifecycle`: 所有・参照・保存済み証跡を検査するDocker資源管理。

### Modified Capabilities

- `image-preview-interaction`: 原本の初回表示・反復安定性と画像下部の制御領域。
- `multi-format-document-preview`: 形式別・状態別の全操作設定、共通Loading、HTML/Office closeの応答性。
- `tab-context-menu`: グループ名の確定・取消と継続操作。
- `release-ci-integrity`: 外部配布同期の対象別結果と安全なHomebrew生成。

## Impact

KatanA UI、画像loader、タブ状態、配布スクリプト、Docker検証と関連テスト。既存checkoutのrelease/v0.22.43で実装し、公開済みv0.22.42のtag/assetを変更しない。KDV/KRRはこの担当で編集せず、必要な上流対応はIssueへ切り分ける。本人の実機確認はIssue Close条件にしない。通常レビュー・自動検査・公開・後処理は維持する。
