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

## 通常hookで検出した追加の期待値不一致

最初の通常pushは実session89511で終了1。公開前のPython検査568件中1件が失敗し、公開はされなかった。
`test_native_test_inventory_wiring.py`がinstallerの個数を3個と固定しており、coverage用を追加した実4個と不一致だった。
全installerが既存の固定SHAであることを要求する検査は維持し、この個数だけ4へ同期した。
また、flow回帰にはinstallとcoverageの順序を明示比較するassertを追加した。

対象wiring8件・flow11件は各実終了0、flowファイルは196行。
元の失敗ログは`tmp/oct24-coverage-installer-normal-push.log`、補正後の対象ログは`tmp/oct24-coverage-installer-wiring-green.log`と`tmp/oct24-coverage-installer-order-green.log`に保持する。
最初のpush失敗を新HEADの合格へ置き換えず、正規hookの結果を改めて回収する。
