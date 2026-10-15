# #352 既存関数長APIの計測修復

対象: [KatanA #352](https://github.com/HiroyukiFuruno/KatanA/issues/352)

## 不具合と修復範囲

`FunctionLengthOps::check_length` は開始・終了とも同じjoined spanの開始位置を使い、行差が常に0になっていた。開始braceのstartと終了braceのendを使う最小修正で、既存の`end - start`という行差と定数30を維持する。free function・impl methodだけという既存の対象を拡張しない。

新規公開API integration `crates/katana-linter/tests/function_length.rs` は、30/31行差、実違反名/位置/計測値、impl、入れ子の独立計測、signature改行非算入、単一行を検査する。文字列を実syn parserへ渡し、実lint APIの診断を検査する。mockや検査除外は追加しない。

## 実検証

- 旧source: 実23101 exit101、7件中4FAIL/3PASS。raw `tmp/function-length-contract-red-2026-10-15.log`。
- 修復後: 実88813 exit0、7PASS。raw `tmp/function-length-contract-green-2026-10-15.log`。
- 正式入口 `just JOBS=1 ast-lint`: 実40531 exit0、30PASS/76filtered、6suites/24.92秒。raw `tmp/function-length-formal-ast-2026-10-15.log`。新7件は既存の`ast_linter` filter対象であり、専用テストだけで終わらない。
- `just fmt-check`初回は新テストのformat差でexit1、公式cargo fmtを適用後の実67524 exit0。`git diff --check`成功。

## 正式ゲートとの区別

通常hookのshared gateは公開registry `katana-ast-lint=0.5.2` を呼ぶ。このKatanA内旧APIとは別実装なので、今回の修復を共有KALの修復や全関数30行遵守の証明に読み替えない。旧APIの全repoゲートへの新規接続、trait/defaultへの対象拡張、閾値/除外変更は行わない。共有側の同不備は診断・Issue引継ぎで分離し、別repoの実装を代行しない。

主査が公開registryの同実装、rule catalog/既定30、既定有効/実行入口を再読して確認し、[KAL #16](https://github.com/HiroyukiFuruno/katana-ast-lint/issues/16)へ診断と受入候補を引き継いだ。KAL source/cacheは編集していない。

通常commit/push・current review・公開確認は、この局所GREENだけでは完了扱いにしない。旧HEADの全coverage・静的macOS packageを変更後HEADの成功へ流用しない。
