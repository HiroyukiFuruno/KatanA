# 検査invocationの時間観測（2026-10-19）

対象: [Issue #343](https://github.com/HiroyukiFuruno/KatanA/issues/343)。実装開始HEAD: 050b42a3。

## 変更

既存の通常テストと4段階coverageの実行記録には、個々のCargoコマンドの経過時間がなかった。
重複削減の導入前後を比較できるよう、既存の一回の実行に観測を付加した。追加テスト実行やskipは導入しない。

- `invocation_timing.command`: Popen直前からCargo終了・stdout消費完了までの単調時計時間とUTC開始/終了。
- `invocation_timing.diagnostics`: Cargo終了後のinventory生成、native listing、source-after取得の時間。
- 比較CLIと既存CI summaryに、通常/各coverageのコマンド時間・診断時間・終了codeを表示する。
- 時間は単調時計を使い、UTC時計の前後差で代用しない。時計補正でUTC終了が開始より前でも、単調時計の値を保持する。
- 欠測・未知schema・非UTC・bool/負数/非有限の時間は未計測と表示し、0秒へ置換しない。
- 固定phaseの欠落/selector不一致、候補比較不可、重複run、合計overflowがある場合はcoverage時間合計を出さない。

計測範囲は全job/runner/CI待機時間ではない。コマンド前の環境取得、JSON保存も含まない。
コマンド時間はbuild/test/出力処理を含み、純粋なtest CPU時間や短縮効果ではない。
実行記録は作者が編集可能な診断であり、`execution_equivalent` / `reuse_authorized` は引き続きfalseである。

## 検証

- 旧実装の実Cargo成功/失敗回帰で新field欠落のKeyErrorを確認: `tmp/oct19-343-timing-red.log`。
- 対象78件成功、23.909秒、実exit0: `tmp/oct19-343-timing-integration.log`。
- 小さな実Cargo workspaceで通常テストと4coverage phaseを実行し、比較JSONとsummaryに時間が出ることを確認。
- 元のfixture skip差、成功0/失敗101、実行同等性/reuse=false、別run欠落の拒否を保持。
- 純粋回帰で欠測、不正時間、時刻補正、phase重複、合計overflowを検証。
- Python AST構文/関数30行以下/各file200行以下、diff検査成功。
- 対象KML成功。独立した最終差分レビューでP0/P1/必須P2は0件。

## 残作業

現行アプリ全体の時間やWindows cold/warmの実測、通常とcoverageの完全な実行同等性、実重複削減、trusted reuse、導入前後の総時間比較は未完了。
今回の小fixture成功をそれらの証明に流用しない。正式pushはKDV #69の安全性修正版採用後に元ゲートを通し、current review・Ready必須CI・packaging・公開を継続する。
