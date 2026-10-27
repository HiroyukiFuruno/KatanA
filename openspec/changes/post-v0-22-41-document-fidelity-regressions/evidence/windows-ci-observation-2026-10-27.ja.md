# Windows CIの観測結果と残件（#343）

## 結論

観測機能は通常commit `ef29a17d5d3da49ea865063e079023cc52a477f5` で実装済みである。
未完了のチェック欄だけを根拠に、未実装と判断していた点を訂正する。
今回、既存Windows CIから記録の実出力を回収し、機能の実装状態と残る測定を分けた。
製品検査の成功、検査削減、結果再利用、公開の完了を示す記録ではない。

## 回収した一次情報

- [CI run 40402573799](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40402573799) は全体cancelled。
- [Windows job 121851995933](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40402573799/job/121851995933) はfailure。
- `test-inventory-Windows` artifact ID `12690785625` は82,841 bytes。
- ZIPのSHA-256は `00227c61c2fa5baa95790b1df194b907acccdb495eee9d2803775d3f704204a4`。
- `windows-timing.json` と `normal.json` を実際に読み、元artifactのdigestを照合した。

GitHubが検査したPRのmerge refは `3289bd904ccc8f4544c33fe532eaf6a41ef25e2e`。
そのtreeとPR HEAD `c3e39dae2197b1960ee5412ab90300f7898ed520` のtreeは同じ
`808c22d093fe2e39f19a5081fac5948c2f00c7c2` である。
現在のローカル修正 `ef94cc6f` のCI成功へ流用しない。

## 実測値

| 項目 | 実際の記録 |
| --- | --- |
| 既存workspace test | 1回実行、386.7617079秒、終了コード101 |
| run / attempt | 40402573799 / 1 |
| sourceとlock | 実行前後clean・同じtreeと両lock、source_stable=true |
| registry cache | job logでmissを確認 |
| compiled target cache | eligible=true、job logでmissを確認 |
| inventory | complete=false、exit_code=101、reuse_authorized=false |

Windows側の両lockのdigestは、GitのblobをCRLFへ変換したdigestと一致した。
LFのローカルファイルとdigestが異なることを、依存更新や入力改変と誤認しない。
cache-hit出力の空文字を観測JSONはnullとして保存しているため、原記録は改変せず、
実job logの `Cache not found for input keys` を別の根拠としてmissと判定した。

## 現在sourceの契約検証

ローカル `ef94cc6f` の既存観測契約4件とobserverの実subprocess回帰10件が成功した。
元のtestを1回だけ実行し、子の不合格と記録失敗を成功へ変換しないことを検証している。
workflow、実装、テストへの重複変更は行っていない。

## 残る範囲

- 対応するcold/warm条件をそろえた比較、短縮効果、完全な同等性、trusted reuseは未完了。
- GitHub画面上のStep Summary表示は直接確認していない。追記実装と契約検査は確認済み。
- 新しいHEADの通常push、current review、全CI、配布起動、全OS公開は未完了。
- Windows検査失敗とmacOSの補助診断採取失敗は別事象として扱う。

詳細値、source/lock照合、選別したcache log行は
[機械可読の回収記録](windows-ci-observation-2026-10-27.json) に保存した。
原ZIPと展開したJSONはignoredの `tmp/oct27-c3-windows-ci-observation/` に保持している。
