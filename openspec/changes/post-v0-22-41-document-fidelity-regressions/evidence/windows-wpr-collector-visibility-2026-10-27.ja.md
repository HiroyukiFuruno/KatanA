# Windows待機診断の記録設定と欠落を保存する

## 観測した不足

source ee91のWindows診断ETLでは、長い待機33件の起床側stackを照合できたが、
待機開始側の同一source QPCのstackは見つからなかった。
これはGeneralProfileのstack設定不足、イベント欠落、別の記録条件のどれによるものか未確定である。
既存artifactに保存されたWPR statusは有効なstack設定やcollector別の欠落を含んでいなかった。

## 修正範囲

既存のnonce付きWPR instanceに対して、アプリ起動前と所有アプリ終了処理後・WPR停止前に
`-status collectors -details`を実行し、stdout/stderr/exit codeを既存の有界native呼出しで保存する。
それぞれ`wpr-collectors-before`と`wpr-collectors-after`で識別し、失敗は診断失敗として残す。
終了時の照会が失敗しても、既存の所有instanceだけのWPR停止は試行する。

元のGeneralProfile、memory mode、アプリ観測10秒、元smokeの200ms×10回、
メモリ・font・実行物照合、既存PID照合と所有instanceの終了処理は変更しない。
独断でcustom profile、driver、renderer、待機閾値や保護設定を変更する修正ではない。

[MicrosoftのWPRコマンド仕様](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options)
は、`status collectors`がSystem Keywords、System StacksとEvents Lostを表示し、
`-details`でcollectorの追加情報を出すと説明している。
CSwitchイベントの存在だけでCSwitch stackも有効だとは推測しない。

## 限定検証と未完

- 元sourceに追加したcollector記録順序の回帰は失敗し、修正後は成功した。
- Windows trace、同一配布物の比較、全OS公開の既存契約と合わせて17件成功。
- 隔離したPowerShell 7.4.2の実Parser.ParseFileは構文エラー0。
- 独立レビューの指摘を受け、終了照会とWPR停止を同じtryにまとめる不正変更を
  拒否する境界回帰を追加した。同変更の実テストはexit1、通常17件は成功。
- 初回のテスト起動に存在しないmodule名を指定して1件の読込エラーが出た。
  実在する`test_all_os_release_contract.py`へ訂正して上記17件の終端成功を回収した。

これはWindows実機でWPR照会を実行した証明ではなく、stack設定や欠落原因の判定、
待機開始APIの特定、元200ms基準の達成、全OS公開は未完である。
次のWindows実行で実際の設定と欠落数を回収し、必要な修復を続ける。
