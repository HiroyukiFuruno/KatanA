# Windows配布版の描画待機を採取済みETLで判別する

## 現在の結論

正式公開run [40313264436](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40313264436)
（source `ee91bc91467b076acd1e225fad15616820c08595`、attempt 1）は不合格。
Windowsの元200ms×10検査は `last_frame=3; current_frame=3` で失敗した。
Linux、macOSの両アーキテクチャの実配布起動は成功したが、Publishと公開後整理は
skippedであり、公開成功ではない。終端後のtag/Release APIも404だった。

本人は200ms条件の例外を不採用とした。元条件、100ms repaint、RSS、フォント、
本体・worker同一性、保護機能を維持する。下記の別診断起動や解析成功は、元の
Windows起動検査を合格へ置き換えるものではない。

## 同じ配布実行ファイルの限定診断

元不合格後の公式WPR診断・同一nonceでの停止・tracerpt変換・artifact uploadは成功。
artifact `12647334398` の44ファイルを回収した。診断exeのSHA-256は
`820962f6ea77c17fbf71a888276c104b2cfd9dce4ecb29ffd4ec73cb2b47a210`。
空の隔離設定、PID 1232、main TID 7216の採取時間は10.186秒だった。
本人の設定や利用規約承認を変更していない。

| 入力 | SHA-256 |
| --- | --- |
| Windows job raw log | `8083041c9bf4630dd95557f3d13acf1f394fe60ec6881a95da4b116e7cb436db` |
| WPR ETL | `2abd693d7e1a1b4ee92ac1d21eeeb2b62da641718dc1740f576e915b1e6e9866` |
| tracerpt CSV | `79d4d5152087e0c071be93057d7eb17ea6af0b3e0d9fd1d18bd34acdd8829f25` |

原smokeの7対、診断の35対のUI begin/endはparser error 0。診断の定常UI処理は
概ね0.5–1.2ms、heartbeat記録は0.2–0.4msでも、UI開始間には概ね301–319msの
空白があった。処理内の数値だけで本人の遅延報告や元検査の不合格を否定しない。

## 待機とCPU割当を区別した結果

tracerpt CSVのCSwitch payloadはDataSizeだけだったため、再起動・再採取をせず、
隔離した解析用環境のdissect.etl 3.14で同ETLのraw payloadを復元した。
製品や配布物の依存関係には追加していない。

Microsoft PerfViewの[公式parser](https://github.com/microsoft/perfview/blob/main/src/TraceEvent/Parsers/KernelTraceEventParser.cs)
のCSwitch/ReadyThread/StackWalk schemaへ照合した。CSwitchは全27,833件、
version 5 / 28bytesでCSV件数と一致。mainの3,528イベントから1,763対を抽出し、
検査対象の重複switch-outは0件だった。byte13は従来WaitMode enumに適合しないため
rawのまま保持し、推測で解釈していない。

200ms以上のoff-CPU区間33件は、すべてstate 5 / reason 6。
これはWaiting / UserRequestを示すが、特定のAPI名ではない。
同ETLのReadyThreadは12,501件で、mainを起こす1,279件すべてにsource QPCが
完全一致するStackWalkがあった。33件の長待機には、それぞれ起床eventが1件あり、
Systemの描画経路5件、dwm.exeの合成処理28件へ厳密に対応した。

| 区間 | 33件の実範囲 |
| --- | --- |
| switch-out → ReadyThread | 279.2688–316.927ms |
| ReadyThread → switch-in | 0.0129–0.0542ms |

したがって、mainがReadyのまま約300ms CPUを奪われた状態とは区別できる。
ただしswitch-outのsource QPCに完全一致する開始stackは33件とも無かった。
待機解除元がgraphics/compositorであることと、開始APIがPresent、ResizeBuffers、
DwmFlush、message waitのどれかであることは同義ではない。近傍のWARP stackを
開始stackやCPU占有時間へ読み替えない。

mainのdisk write completionは660件でpartition応答の合計85.747ms、最大4.6569ms。
代表289.008msの空白内では2件、合計0.287msだった。単一disk書込み300msという
説明は支持されないが、これをWriteFile全体の壁時間の測定とは呼ばない。

## 変更候補の境界

- eframe 0.36.2のNativeOptionsは既にLOW_LATENCY、AutoVsync、latency 1である。
  設定漏れという仮説は除外した。
- 選択済みwgpu/core/hal/typesは全て30.0.1。AutoNoVsyncの実fallbackは
  Immediate → Mailbox → Fifoで、無条件採用はtearingを選ぶ場合がある。
  これを安全な修復として採用していない。
- 現eframeのFrame/CreationContextはRenderStateのcloneを持つ。調査した0.36.2では
  Frame側surface configの変更を実Painterへ戻す同期が無く、単なるfield変更や
  selector単体テストでは実効性を証明できない。
- 公開0.37.0には実Painterへの伝播がある。ただしegui系、winit境界、既存vendorの
  表示修正の維持が必要で、更新済み・互換・Windows合格とは扱わない。
- 同じadapter/backendで、対応している非tearing modeを実surfaceへ適用できる場合は、
  元検査をそのまま使って検証する候補になる。未対応mode強制、driver変更、閾値免除で
  成功扱いにしない。

## 保存した一次証跡

### 次の自動比較

Windowsの元検査が失敗した場合だけ、同runで生成したZIPとidentityを保存し、
Windows Server 2022へ渡して比較する経路を追加する。再ビルドはせず、source、
run/attempt、archiveのsize/SHA、ZIP内main/workerのSHAを先に照合する。
旧ee91の実行ファイルを取得できたという意味ではなく、次のrunの同一bytesを
両hostで用いる。実SHAと結果の回収前に同一性・高速化を主張しない。

元Windows PowerShell検査本文をそのまま抽出して実行する。本文SHA-256は
`726979a6a0eb68c1eb8c64f4b277c4fef923d9f67043b21469caa376b7ac7cd7`。
同じ200ms×10/RSS/font/identity/worker/終了処理を使い、pass/fail両方の実ログと
host情報を保存する。driver/backend、利用規約、本人設定を変更しない。

現在の必須Windows jobはwindows-latestのままで、Publishの依存条件も変更しない。
Server 2022比較は公開を実行せず、元2025不合格の代わりにもならない。
host差が判明した場合も、2025修復済み・全coverage不変とは扱わず、実結果を
根拠に次の修復・参照環境方針を検討する。現時点で比較の実動作は未検証。

### ローカル保存先

作業checkoutの`tmp/oct26-ee91-windows-startup-diagnostics/`、
`tmp/oct26-ee91-windows-phase-measurements.json`、
`tmp/oct26-ee91-raw-cswitch-result.json`、
`tmp/oct26-ee91-raw-cswitch-review.md`、
`tmp/oct26-review-etl-ready-thread.py`にraw・対応表・再現手順を保持した。
大容量ETL/CSVや解析用依存を配布物へ含めない。

必要修復、同HEADの通常review/CI、Windowsの元基準成功、全OS公開・後処理は未完。
