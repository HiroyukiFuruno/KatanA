# Windows表示設定の期待値をOS別にコンパイルする

関連: Issue #364、PR #370。

## 検出した不合格

HEAD d875cc2のCI40476315373は7/8 job成功、Mac job122095306188のcoverageだけが失敗した。唯一の未実行行は `native_graphics.rs:23` のWindows用テスト期待値代入だった。`if cfg!(target_os = "windows")` はMacでも構文として存在するため、LLVMの行計測に0回の分岐が残った。製品テストは成功しており、MacのUI並列145件（既存ignore2）、serial18件も成功していた。Readiness40476315446も成功した。これらを新HEADの全ゲート成功へ流用しない。

## 最小の修正

変更は `#[cfg(test)]` 内だけ。期待するpresent_modeをOS別の `#[cfg] let` へ分ける。WindowsはAutoNoVsync、他OSは既定値の比較を保ち、最大待機frame1・centered/multisampling/depth_buffer/dithering/run_and_returnの全assertも維持する。

製品部分（`#[cfg(test)]` より前）は旧HEADとbyte一致し、SHA-256は `f4ca326c6ab19abc7f021f69d1793970356038b3a934b9a5bd3256177a9f8149`。coverage設定・除外・閾値・製品の表示方式・元Windows200ms×10には変更しない。

## 対象検証

実アプリbin `KatanA` の同名テスト1件を、公式ゲートと同じ `--cfg coverage` を使ってLLVMで実行しexit0。native_graphics.rsの16/16行、21/21領域、2/2関数が100%。これは当該ファイルのMac対象検証であり、全workspace coverage・Windows実行・実配布起動・公開の証明ではない。

記録: `tmp/oct27-native-presentation-cfg-coverage-receipt.json`、JSON report/log、`tmp/oct27-native-presentation-cfg-test-change.json`。初回のCLI引数/対象名誤りはコンパイル前に失敗し、正しい `--bin KatanA -- <test-name>` の実exit0と区別する。

rustfmt/diff検査と独立レビューP0/P1/P2なし。対象Clippy（実test target含む、16.05秒/exit0）も成功。新通常統合・push hook・exact current review・全CI・実配布/公開は別に回収する。OS条件だけの期待値はcompile-time cfgへ束縛し、実行できない他OS行を足さない。既存100%ゲートを再発検出として維持する。
