# 参照候補の状態訂正と独立修正の横断検証

## Issue #338 の切り分け

10月15日のproducer `9aa70da50781a676867dfed06b440ac96d6e3208` は公開registry KRR0.4.23/KDV0.5.12を使い、正規runnerで1280×2400候補を生成し、宣言検証とKDV担当への引き渡しを行っていた。
公開証跡: [10月15日の候補生成と引き渡し記録](https://github.com/HiroyukiFuruno/KatanA/issues/338#issuecomment-6192691970)。

10月19日に保持されている画像・geometry・宣言・source identity・fixture・request・logのSHA、Git上のmanifest/両lock、frame1223とcrop寸法を再照合した。旧runner/workerバイナリは現在存在しないため、その歴史的digestを再測定したとは主張しない。

既存KDV担当のturn `01a15399-c6df-7661-ab2b-630c3d54e37f` はcompleted。入力の存在・SHA・寸法の一致を確認し、追加producer実装不足の根拠はない一方、採点・参照採用済みの記録は確認できないと回答した。追加ビルドは行っていない。

`tmp/issue338-9aa70da5-capture-2026-10-15/verification.json` はprovenance_verified=trueだが、visual/semantic/interaction/performanceは全てnot_evaluated。生成・宣言検証・handoffは実施済み、採点・参照採用は未完了として分離する。成果物確定が残るためpartial/verification-pendingとOPENを維持する。起票時0.4.19へのdowngrade、参照上書き、95点達成扱いはしない。

## 独立5コミットの横断検証

対象HEAD: `59c5207880fa68bd8f1111dc13b6561829c7ef19`。検証開始・終了時とも同じHEADかつtracked clean。

既存preflightと同じ検出範囲を一度実行した。

```sh
rtk proxy python3 -B -m unittest discover -s scripts/release -p 'test_*.py'
```

実行session68539はactual exit0、474件/197.744秒/OK。raw logは `tmp/oct19-59c-release-python-all.log`。途中のreference-transaction拒否stderrは負例fixtureであり、最終結果と区別する。

共有 `.git/config` の前後SHAは `bcf708315eb9ae84c706129108d49862ef47e24a0d13a615f0b3bd58e0fb3c29`、core.bare=falseのまま。実repo hook/config変更はない。

これは独立修正同士のPython回帰検証であり、全preflight・供給網・Rust・Windows実機・CI・packaging・公開成功の証明ではない。通常pushは既知のKDV #69供給網問題で未通過のまま。迂回や同一失敗の反復pushは行わない。
