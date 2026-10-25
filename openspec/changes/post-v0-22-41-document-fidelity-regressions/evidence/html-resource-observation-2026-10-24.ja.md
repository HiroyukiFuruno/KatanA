# HTML表示状態の読み取り専用記録

## 範囲

既存のDEBUG・出力先・run IDの対指定でだけ有効になる資源記録へ、HTMLのURI、KatanA内adapter識別子、受理frame境界、実frame generation、viewport、scroll、content height、idleを追加した。通常OFFは観測・URI clone・HTML走査を行わず、既存100ms gate後だけ採取する。

adapter識別子は実adapter起動後にだけ採番する。受理frame境界はKatanA内の局所値であり、KRR内部UUIDや個別navigation commandとの厳密な因果関係を証明しない。KDVはrequest tokenを公開していないため、同URIの旧queued frameを個別commandへ完全に帰属させられない限界を保持する。

未起動・未受理は観測なし、非有限geometryは記録失敗とし、欠測値を0で埋めない。未保存HTMLのsource hashは未実装であり、per-frame hashを追加していない。

## 実検証

- 初期実装の対象2件: `tmp/oct24-html-receipt-observation-tests.log`、実終了0。
- 資源記録10件: `tmp/oct24-html-receipt-resource-tests.log`、実終了0。
- 厳格Clippy・fmt: `tmp/oct24-html-receipt-lint-impacted.log`、`tmp/oct24-html-receipt-fmt-check.log`、各実終了0。
- 独立レビューは新getterの30行上限違反を検出し、private helperへ分離して修復した。
- 修復後の主査対象2件: `tmp/oct24-html-receipt-main-matched-tests.log`、session22995実終了0。先行の別profile/toolchainによるcold buildは所有CargoへSIGINTを送り実130で回収し、検査成功へ読み替えていない。
- 独立再レビュー: `tmp/oct24-html-receipt-independent-review.md`、修復後P0/P1/P2なし。
- 最初の通常commitはASTがproduction file246行を検出して実終了1。`tmp/oct24-html-observation-normal-commit.log`と元RTKログを保持し、閾値を変えずadapter session責務を専用moduleへ分離した。分離後対象2件はsession24443実終了0、`tmp/oct24-html-session-split-tests.log`。最終独立レビュー `tmp/oct24-macos-isolation-final-review.md` は本体196行/session52行・P0/P1/P2なしを確認した。

配布版本体の操作・描画・正常終了、通常OFF性能、独立95点の完了とは別である。macOSの最新配布生成と隔離実検証を続ける。
