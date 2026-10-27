# 補助診断の採取成功テストを非必須ジョブへ分離する

## 本人の承認範囲

2026-10-27、本人は「その採取成功テストだけ診断専用の非必須ジョブに分離」し、
本体の必須検査と基準を維持してリリースまで継続するよう指示した。
採取成功を製品検査の成功に置き換えず、診断の失敗も診断として保持する。

## 実装

- 既存の実採取成功テスト1件だけを、非default feature `native-profiler-diagnostics` へ分離した。
- 実採取テストの本体、PIDとThreadの出力確認、元5秒制限は変更していない。
- 通常側のopt-in・コマンド・5秒境界の3テストは引き続き実行する。
- `just test-native-html-diagnostics` はmacOS上でその1件を明示的に実行する。
- 別workflow `HTML Failure Diagnostics` の非必須ジョブで実行し、Draft PRではrunnerを起動しない。
- 診断ジョブの失敗を成功へ変換するcontinue-on-error等は追加していない。
- 診断レポートはビルドcacheの対象外とし、以前の採取結果を新runのartifactへ混在させない。

元HTMLテスト失敗時の有界・opt-inの採取と `sample_error` の追記は維持する。
本体のworkspace test実行部分はbyte単位で同一である。
3OSのtest/lint/coverage/供給網/packaging、HTML描画の元条件、Windows200ms×10は変更しない。
GitHubの既存required checks7件も変更しておらず、診断ジョブはその一覧に含まれない。

## 変更を保護する検証

契約を追加した旧sourceで2失敗・1エラーとなり、実装後は6件成功した。
元の製品CI resource契約10件、actionlint、通常側の実テスト3件も成功した。
実採取テスト本体と製品のRun tests部分にはSHA-256による変更検出を加えた。
専用feature、実行入口、Draft guard、レポートのcache外配置も検査する。

このホストの実採取成功は未確認であり、成功したと報告しない。
専用ジョブは新HEADのReady後に独立した成否を回収する。
新HEADの通常push、レビュー、全製品CI、配布検査、全OS公開は別の未完工程である。
