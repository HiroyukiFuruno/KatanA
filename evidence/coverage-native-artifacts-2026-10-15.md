# Issue343 coverage実行のnative artifact収集

対象: [Issue343](https://github.com/HiroyukiFuruno/KatanA/issues/343)

## 変更範囲

inventory有効時のcoverage四phaseについて、cargo-llvm-cov 0.9.1の公式external-test方式（show-envでinstrumentation設定→通常Cargo test→既存report）を使い、実testと同じ実行のCargo JSON compiler-artifactを収集する。`--no-report`と`--no-cfg-coverage`だけをinstrumentation設定へ移し、workspace/package/native test選択・threads・skipは保持する。元のcoverage.sh四phase、clean、Office worker、report、除外、閾値は変更していない。

show-envをshell evalせず、既知キーのexport/unsetだけを有界metadataとして解析する。wrapperによるinstrument-coverageが存在しない、重複・未知・不正な出力は拒否する。親環境を変更せず、選択した計測環境を実testとnative listingへ共通に渡す。

## 実際に修復した失敗

実tinyCargoの最初の2件はcoverage artifactが空でassert失敗（40889）。instrumentation接続後は実testとnative listingが成功したが、生成profileとreport探索先が食い違いreport失敗（90613）。0.9.1のshow-envがCargo metadataのtarget/buildを出力する仕様を確認し、既存llvm-covの保存先をCargo metadataにも合わせた。通常のreport呼出環境でもprofrawを発見し、実際にcovered linesが正になることを検査する。

## 検証

- 49302 actual exit 0: 実instrumented Cargo testとprofile/report、失敗test終了101保持の2件。
- 36360 actual exit 0: native inventory関連42件、7.257秒。raw `tmp/coverage-aligned-suite-final-2026-10-15.log`。
- 57273で別build先指定と元reportの不一致を実再現。0.9.1がCargo build-dir設定のある場合だけ別build先を使う条件に合わせた。42406 actual0/43件8.362秒、設定済みbuild-dirも追加した4029 actual0/44件9.889秒。最終raw `tmp/coverage-configured-build-final-2026-10-15.log`。
- 実native正例・ignored一覧・失敗終了101・通常target・空白付きpath・warm再実行（初回fresh=false/再実行fresh=true）・同じexecutable・元環境からのcoverage JSON report（covered lines > 0）を確認した。mockは使っていない。
- warm追加途中のmacOS `/var` symlink表記比較、Cargo fresh属性比較の失敗はtest期待の補正であり、製品修復のREDへ加算しない。旧ログは保存した。
- 新/主要変更Pythonの関数30行以下をASTで確認。
- 独立read-onlyレビューのsplit build P1候補は、一次ソースと実reportの43/44件を照合して撤回された。自己レビューでは元4phase/終了code/元report環境/親環境非変更/非再利用の境界を確認した。日本語WHYは最新本人規約に従う。

## 未完了の境界

これは計測用diagnosticの改善であり、normal testとcoverageの同値性、全workspace coverage成功、時間短縮、Windows native listing、doctest inventory、実test時のbinary digest、CI上での再利用証明ではない。`complete=false`/`reuse_authorized=false`を維持する。検査のskipやreuseは追加せず、Issue343・Ready・全配布・公開の完了を主張しない。

## 正式統合とCI実行環境

source/evidence/tasksは通常署名付きcommit `aead4b6807a71a9efee2c1dc2a3873df8ad3e7db`へ統合。通常push3158はnative→Linux実container→preflightを通りactual exit 0、local/origin/PR346の同HEADとahead/behind0/0を確認した。raw `tmp/coverage-native-artifacts-normal-push-2026-10-15.log`。これはReady後の3OS/全coverage/配布検証ではない。

caller監査でpreflightのdiscoveryが実coverage回帰も実行する一方、Readiness/CD preflightに必要tool/componentが無いことを発見した。先に両jobの契約を追加し実assert2件RED（`tmp/coverage-preflight-tools-red-2026-10-15.log`）。両jobでllvm-tools-previewとcargo-llvm-cov0.9.1を明示導入する。Draft条件・元preflight・四coverage phaseを維持し、skipしない。変更後45件/10.175秒とrelease flow10件成功。最終raw `tmp/coverage-ci-setup-final-2026-10-15.log`。クラウド上での実install/実行はReady後の別確認とする。
