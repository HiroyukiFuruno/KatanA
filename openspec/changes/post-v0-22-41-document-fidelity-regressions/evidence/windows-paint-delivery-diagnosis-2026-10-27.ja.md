# Windows の再描画要求から次の画面処理までを切り分ける

## 今回の配布版で確認した結果

正式公開 [run 40505473053](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40505473053) の実ソースは `67310328d05c317ce2c18b7362846a429a6084f0`。3OS のビルド、Linux の配布起動、macOS の両 CPU の配布起動は成功した。Windows の配布起動は元の 200ms 観測で frame 5 から進まず不合格となり、公開と後処理は実行されなかった。

Windows の後続記録は frame 9 まで進んだため、恒久停止とは断定しない。UI 内部の処理は数 ms で完了している一方、開始間隔は主に約 300ms だった。たとえば前画面の再描画要求 `13:47:06.443637Z` から次の `RepaintNow` `13:47:06.732914Z` まで約 289ms ある。100ms の再描画予約だけを原因と決めることもできない。

`AutoNoVsync` 候補はこの実検査に合格しなかった。設定変更を改善済みとは扱わず、別の Server 2022 で同じ配布物が検査を通過した結果も、元の Server 2025 の成功へ置き換えない。

## 失敗した検査と追加診断を区別する

元の失敗プロセスは PID 9264。一般名の `heartbeat.txt` は失敗時点の frame 7、`completed-heartbeat.txt` は終了処理時の frame 9 の記録である。例外発生時の観測値 frame 5 と、その後にコピーした値を混同しない。

WPR はその後に別の診断専用プロセス PID 7544 を起動して採取した。同じ実行ファイル SHA-256 `6c371a41de855c92da348cace1843532f5b99971f9851a1421ba949a6757839e` を使い、専用の heartbeat は frame 34 だった。34 回の開始記録の間隔の中央値は 312.634ms。約 300ms の間隔は 10 秒間の観測中にも継続したが、この別実行を元の失敗プロセスそのものの stack と説明してはならない。

新しい ETL の dropped event は 0。診断プロセスの main thread 4344 には 200ms 以上の off-CPU 区間が 32 件あり、中央値 309.341ms。各区間に対応する ReadyThread と、起床側の source-QPC に一致する stack が確認できた。起床側は `dwm.exe` が 27 件、`System` が 5 件だった。一方、待機開始時の main thread stack は完全一致する記録を確認できず、待機 API や根本原因は未確定である。起床側の stack を待機開始 API の証明にしない。

## 次の検証と保持する条件

同一の配布バイナリと元の起動検査を使う、公開判定とは別の限定診断を追加する。最初に詳細な native trace の有無だけを変えて比較し、診断ログ自体の負荷が結果へ影響していないかを調べる。各プロセスと設定を隔離し、実行ファイルの同一性、元の 200ms 観測 10 回、RSS、font、worker の条件と終了処理を保持する。

診断用の成功は正式公開の成功ではない。元の失敗を保持し、原因を確認したうえで必要な KatanA の修復、通常レビュー、必須 CI、全 OS の実配布検査と公開へ進む。KDV/KRR 待ちや本人確認待ちへ戻さず、閾値や保護を緩和しない。

## 診断入口の対象検証

新しい派生 script の契約 9 件、既存の同一配布物比較契約 6 件、既存の非必須 native 診断分離契約 7 件は成功した。Actionlint 1.7.12 による変更 workflow の検査も成功。workflow の PowerShell 本文 4 件、生成した比較 script 2 件と解析入口を、既存の隔離 PowerShell 環境の実 parser で読み、構文エラーがないことを確認した。これは Windows の OS API 実行や実比較結果の代わりではない。

Controller の checkout と配布物 source の checkout を分離する。今回のような PR closed 起点では API 上の run head が PR head でも、配布物の source はマージ後の commit となる。失敗 run の親関係と、artifact の run、attempt、source、ZIP、main、worker の同一性を照合する。各比較の出力先は検証済み原本と別にし、元 script と metadata を上書きしない。

静的な独立レビューでは対応が必要な P0/P1 はなかった。通常 commit、通常 push と実 Windows 比較結果の回収は別工程として継続する。未確認の window style 変更や driver 変更は今回の比較へ追加しない。

## 証跡の所在

生ログと解析結果は作業台帳の `tmp/oct27-67310328-build-windows-job-122214725895.log`、`tmp/oct27-67310328-windows-native-failure-summary.json`、`tmp/oct27-67310328-windows-wpr-analysis.json` に保存した。WPR の生 ETL と CSV は同じ run の artifact `12739101004` から取得し、API digest と ZIP の hash を照合した。公開時には正式な配布物と checksum の別の取得記録を必要とする。

## 詳細ログの有無を変えた実比較

診断 [run 40524518173](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40524518173) は、意図どおり macOS 診断を起動せず、Windows だけで同じ配布物を実行した。実行ファイルの SHA は上記と一致し、診断 artifact の取得 ZIP も API digest と一致した。

| 条件 | 元 200ms 検査の結果 | 不合格までに記録した開始間隔 |
| --- | --- | --- |
| 元の詳細ログ | frame 4 から進まず不合格 | 340.754、297.345、298.847ms |
| native trace を低減 | frame 3 から進まず不合格 | 377.002、312.121ms |

この 2 回ではログを減らしても改善しなかった。少数の区間の中央値を一般的な性能値や、すべての原因の確定に使わない。元の正式 run の不合格と未公開は変わらない。結果の正本は `tmp/oct27-40524518173-receipt.json` と取得済みの各 case の生ログである。

## 描画ライブラリの待機方式を比較する次の範囲

現在の egui-wgpu 0.36.2 は backend options を既存環境設定から作り、lock で解決した wgpu-types 30.0.1 は `WGPU_DX12_USE_FRAME_LATENCY_WAITABLE_OBJECT` の `Wait`、`DontWait`、`None` を扱う。wgpu-hal 30.0.1 では、既定の `Wait` は handle を取得して acquire 前に待ち、`DontWait` は handle を取得するがその明示待機を行わず、`None` は handle の取得も明示待機も行わない。

実 backend は swapchain の waitable-object flag を無条件に設定しており、`None` の API 説明だけから flag が作られないとは説明できない。ケース名は設定値どおり `dx12-none` とし、比較で確認していない内部機構の変化を推測しない。

同一配布物の追加診断で、この設定だけを変え、前後に既定条件を実行して順序や初回キャッシュの影響を区別する。これは描画同期の比較であり、OS の保護や graphics validation を無効にするものではない。元の 100ms 再描画要求、200ms 観測 10 回、frame latency 数、RSS、font、worker、identity、正式公開条件を変更しない。

この時点では追加診断の実結果は未回収で、製品の描画設定へ採用していない。API の選択肢があることと、今回の待機開始 API の特定や根本修復ができたことを混同しない。

追加した比較入口の対象契約 13 件、Actionlint、生成した 6 ケース・workflow 本文 4 件・分離した PowerShell 入口 3 件と parser 自身の計 14 script の実構文解析は成功した。通常 preflight の `test_*.py` discovery から同じ 13 件を一度だけ実行する入口も追加した。構文解析は隔離した PowerShell で行い、Windows 上の描画結果とは区別する。

Controller は dispatch 時の不変な commit を checkout し、GitHub token は run の照合 step だけに渡す。診断を実行するアプリの環境へ token を引き継がない。Windows の結果 artifact が存在しない場合は診断 job を不合格とし、前後の既定条件を含む 4 ケースの失敗を隠さず回収する。
