# カバレッジ検査ツールの取得を固定

CIの`Run coverage`だけが`cargo-llvm-cov`の最新版を直接取得していた。
他の正式入口が採用する0.9.1と一致せず、同じソースでも検査ツールが変化する経路だった。

既存のSHA固定済み`taiki-e/install-action`を同じmacOS PR条件で呼び、`cargo-llvm-cov@0.9.1`、`fallback: none`を指定する。
旧curl取得だけを除去し、`llvm-tools-preview`の追加、`JOBS=1 just coverage`、観測出力先、coverage閾値・対象・必須CI条件は変更しない。
Draft時には従来どおり親jobが起動しない。配布物のビルド方式も変更しない。

## 対象検証

- 新回帰を旧workflowへ適用すると、最新版の取得を検出して11件中1件失敗、実終了1。`tmp/oct24-coverage-installer-red.log`。
- 修復後は同じ入口で11件成功、実終了0。`tmp/oct24-coverage-installer-green.log`。
- 回帰は固定SHA・版・fallback、installの順序、installとcoverageの同一条件、旧curl不在、既存llvm-tools操作の維持を確認する。
- 既存の公式checksum検証済みactionlint1.7.12で変更workflowの実終了0。`tmp/oct24-coverage-installer-actionlint.log`。
- `git diff --check`成功、既存契約ファイルは195行で200行制限内。

以上は取得方式と配線の対象検証であり、変更後HEADの実CIカバレッジ測定成功ではない。
通常push・新HEADレビュー・Ready後の必須CIで結果を別途回収する。配布版本体の未実装の文書操作記録も解決したとはしない。
