# 隔離プロファイルから通常環境への参照漏れを修復

## 実行前に確認した問題

既存の `KATANA_CONFIG_DIR` はsettings/workspaceへ適用される一方、永続cache・HTTP画像cache・global Markdown lint設定は通常の利用者ディレクトリを参照していた。macOS起動時にはHomebrewのunpin/untap処理も無条件に開始していた。実ソースで確認し、本人アプリ・設定・Homebrewへの操作は行っていない。

## 最小修復

- 非空の既存 `KATANA_CONFIG_DIR` だけを明示profileとし、永続cacheを `<config>/cache`、HTTP画像cacheをその配下へまとめる。
- global Markdown lint設定は同profile配下の `.markdownlint.json` を使う。
- 明示profileではHomebrew整理threadのspawn前にreturnする。
- 変数未設定・空文字の通常起動では既存の保存先と整理条件を維持する。sandboxの無効化、操作用test port、承認済み設定のコピーは追加しない。

## 検証

- platform resolver 2件、core cleanup条件2件、UI lint設定1件が各実終了0。
- 生ログ: `/tmp/katana-isolated-profile-platform-green.log`、`/tmp/katana-isolated-profile-cleanup-green.log`、`/tmp/katana-isolated-profile-ui-green.log`。
- 新helperをまだ定義していない時点の初回REDはcompile/API不在の確認であり、通常環境への実書込やHomebrew変更を再現した結果ではない。
- macOS以外/coverageで未使用定義が残らないようcfgを限定し、core checkと厳格Clippyを実終了0で確認した。`/tmp/katana-isolated-profile-cleanup-check.log`、`/tmp/katana-isolated-profile-cleanup-clippy.log`。
- 独立最終レビュー `tmp/oct24-macos-isolation-final-review.md` はP0/P1/P2なし。

配布版本体による実プロファイル分離・文書描画・正常終了はこのsource検証とは別に続行する。Windows実測や全clean-machine受入を済扱いしない。
