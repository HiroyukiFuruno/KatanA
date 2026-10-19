# Windows native inventoryの取得経路（2026-10-19）

対象: [Issue #343](https://github.com/HiroyukiFuruno/KatanA/issues/343)。実装開始HEAD: 88e910c0。

## 未実装だった点

従来のcollectorはPOSIX selector pipeだけを許可し、Windowsのnative一覧を常に拒否していた。
このままではWindows通常testの実行記録を使って対象集合を比較できない。

## 実装

- POSIXの既存selector経路は変更しない。
- WindowsではPython3.12以上の`os.set_blocking(fd, False)`と`os.read`を使う。
- stdout/stderrを個別にdrainし、片側EOFで他方の出力を捨てない。
- would-blockとread進捗を区別し、空pipeでbusy spinしない。
- 既存の各stream1MiB・64KiB chunk・30秒deadline・parse/raw/hash/argv・終了codeを維持する。
- 上限超過・timeout・読み取り失敗は既存callerのkill/waitへ返す。reader thread、無制限communicate、disk spoolは導入しない。
- 未対応platformと古いWindows Pythonは、成功へfallbackせず診断失敗にする。
- Windows CIにPython3.14系列と、実pipe4件/実Cargo recording3件のfocused回帰を接続した。元Draft guard、3OS全test、coverage、品質条件は維持する。

## 一次情報

- [Python3.12 os.set_blocking](https://docs.python.org/3.12/library/os.html#os.set_blocking): Windows pipe対応は3.12で追加。
- [Python selectors](https://docs.python.org/3/library/selectors.html): Windowsではsocketを扱うがpipeを扱わない。
- [CPython3.12のread実装](https://github.com/python/cpython/blob/3.12/Python/fileutils.c): `_Py_read`でWindows nonblocking pipeのERROR_NO_DATAをEAGAINへ変換する。
- [setup-python v7.0.0](https://github.com/actions/setup-python/releases/tag/v7.0.0): 公式tagのcommit `5fda3b95a4ea91299a34e894583c3862153e4b97`へ固定。公式Windows installerのpython3.exe alias生成も確認した。

## 実結果

- 配線回帰: 旧collectorで1FAIL。`tmp/oct19-343-windows-reader-wiring-red.log`。
- 初回統合はhelper名不整合でactual1（40件/4FAIL/8ERROR）。`tmp/oct19-343-windows-reader-integration.log`。これを成功にしない。
- interface、would-block busy spin、実際にはstdoutを閉じていなかったEOF fixtureを修復。
- 最終対象83件成功、20.402秒、実exit0。`tmp/oct19-343-windows-reader-final83.log`。
- 実子processのstdout/stderr、nonzero7、exact1MiB、stderr超過、片側EOF、stdin待機中timeout、callerによるreapを検証。ResourceWarningもエラーとして実行した。
- Windows CI配線契約は旧1FAILから修復後38件成功。`tmp/oct19-343-windows-native-ci-final.log`。
- default actionlint v1.7.12成功、実exit0。`tmp/oct19-343-windows-native-actionlint.log`。
- 対象Python構文/関数30行以下/各file200行以下、KML、diff検査成功。reader/collectorと追加CI接続を独立最終レビューし、P0/P1/必須P2は0件。

## 未検証

上記の実process検証はmacOS上で行った。同じ標準APIを使うコードでもWindows実機の成功証明ではない。
Windows native取得の実成功はReady後の該当CIで回収する。Draft中のCIは起動していない。
Windows cold/warm、通常/coverageの完全同等性、実際の重複削減、trusted reuse、配布受入・公開は未完了である。
