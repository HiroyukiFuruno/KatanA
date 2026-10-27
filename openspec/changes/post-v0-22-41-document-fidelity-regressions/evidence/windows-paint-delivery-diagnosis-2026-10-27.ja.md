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

## 待機方式を変えた実比較の結果

通常 push の全検査を通過した `bd4ff7f6` を controller とし、[run 40534985776](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40534985776) で 4 ケースを実行した。すべて同じ元配布物の実行ファイル SHA と一致し、取得した診断 artifact `12746794327` の SHA も GitHub API digest と一致した。

| 実行順と条件 | 元 200ms 検査の不合格時の観測 |
| --- | --- |
| 既定条件・前 | frame 6 から進まず不合格 |
| `DontWait` | frame 3 から進まず不合格 |
| `None` | frame 4 から進まず不合格 |
| 既定条件・後 | frame 4 から進まず不合格 |

この比較では修復できなかったため、製品へ待機方式の変更を採用しない。起動から終了処理までのログには後続 frame も含まれるので、それらを元の 200ms 観測で進行した証拠へ混ぜない。元の正式公開の不合格は保持し、Windows の paint 配送と待機開始の識別を続ける。生ログと case ごとの hash・実結果の正本は `tmp/oct28-40534985776-diagnostics/`、取得と解析の receipt は `tmp/oct28-40534985776-receipt.json` に保存した。

## 待機開始の追加採取

既存 ETL の解析では、32 区間の target thread の exact-QPC switch-out stack を確認できなかった。解析の誤結合を示す証拠はなかったが、記録自体に stack がなかったとも確定しない。起床側の stack だけで待機 API を決めず、明示的な追加採取を行う。

非必須の `wait-stacks` 選択時だけ、元の起動検査と結果保存の後に、同じ実行ファイルの別所有 process を短時間記録する。呼出し前に source/run/attempt/main hash を再照合する。既存の `GeneralProfile` は既定のまま、新しい `-WaitStacks` は repository 内の固定 WPR profile のみを使用する。CSwitch/ReadyThread の stack、ProcessThread/Loader の keyword を指定し、profile 本文・hash・WPR が実際に選んだ profile の status を保存する。

既存の記録がないことの確認、固有 instance 名、所有 PID/実行ファイルの確認、timeout と終了処理は維持する。任意の profile、default instance の停止、OS 保護の無効化は追加しない。追加採取の成功を本体の元 200ms 検査の成功にせず、採取失敗は別の error として記録する。

追加入口 14 件と trace 契約 10 件、変更 workflow の Actionlint、隔離 PowerShell の実構文解析と固定 XML の準備・hash 照合は成功した。これらは Windows 上の WPR による schema 受理や、必要な stack が実際に取得できた証明ではない。実採取・原因確認・製品修復・公開は後続工程として継続する。

## 明示採取の実結果と推論の訂正

[run 40542795332](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40542795332) では、元の起動検査は frame 4 から進まず不合格だった。別 process の追加採取は成功し、WPR は要求した `KatanAWaitStacks.Verbose.Memory` を選択、start/status/stop/CSV 変換は終了コード 0 だった。profile 本文と保存 hash、実行ファイル hash、取得 artifact の GitHub API digest を照合した。記録の lost events は 0。

PID 8964 の初期 thread 7568 に対応する長い待機区間は 37 件。switch-out の exact-QPC に target の stack は見つからなかったが、switch-in の exact-QPC には全 37 件で同じ PID/TID の kernel と user stack があった。取得した履歴を、起床側の別 process の stack と混同しない。正本は `tmp/oct28-40542795332-receipt.json` と `tmp/oct28-40542795332-switchin-stacks.json` に保存した。

ImageLoad が記録した timestamp と SizeOfImage に対応する DLL を Microsoft の公式 symbol server から取得し、export とアドレスを照合した。先頭は `NtWaitForSingleObject` と `WaitForSingleObjectEx` に対応する。KatanA の実行ファイルも元 artifact から hash 照合して抽出し、32 件の呼出し復帰先が DX12 surface の取得時の待機、初期 5 件が fence 待機のコード形状・静的な source location と対応することを確認した。ただし一致する PDB はないため、application callsite の source 行を symbol 化して確定したとは扱わない。

以前の「`request_redraw` から次の `RepaintNow` のログまでを GPU 外の遅延とみなす」推論は撤回する。eframe の `RedrawRequested` は `run_ui_and_paint` を実行してから `handle_event_result` のログを出すため、その間にも GPU の surface 取得や描画待ちを含み得る。短い UI 処理時間だけで Windows の message 配送側へ原因を決めない。

次は同じ配布物の前後既定条件と、公開 API の `WGPU_DX12_PRESENTATION_SYSTEM=DxgiFromVisual` を比較する。採取と元の起動検査を分離し、環境値・source・実行ファイル・元閾値を束縛する。実結果は未回収で、製品設定へは採用していない。driver や window style、OS 保護、元品質条件は変更しない。

この比較入口は対象契約 17 件、Actionlint、生成した smoke と分離入口の実 PowerShell parser、最終独立レビューを通過し、通常 commit `a5ffd846` へ統合した。既定前と Visual の元 smoke を先に保存してから、同じ exe の別所有 process を採取する。採取失敗も別項目へ保存して後続ケースを継続し、環境値を `finally` で復元する。既存のログ比較、待機方式比較、無引数の wait-stack 診断は維持する。実 Windows の比較結果は通常 push 後に別途回収する。

## 2026-10-28 presentation比較の実結果と解析訂正

[比較40551477572](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40551477572)は同じ配布物・実行ファイルで既定前/Visual/既定後を検査し、元200ms条件でそれぞれframe4/2/4の進行停止を検出した。Visualへの変更は採用しない。独立した両WPR採取は成功し、各32長待機の再開側に32件の対象stackがあり、lost eventは0。中央値は既定約311ms/Visual約306msで、改善の証明ではない。artifact12753909883のZIP SHA256は`c6509fdaed63dcb47a6ba4a2fcfde9ccc3e7c1584184fa7a2c0dcaa632f1a80e`、controllerは`f9b21d79a6ee274547e2b9d616bbdea8d2f02f87`。元Release・全OS公開の不合格は維持する。

旧ETLを再解析したところ、以前の記録にも32件すべての再開側stackが存在した。最初の解析でswitch-out側だけを検索して見落としていたため、「追加採取がなければcall chainを取得できなかった」という解釈は撤回する。新規採取の37待機とAPI照合結果は保持し、待機開始・再開の両境界を照合する。実行ファイルにPDBがないため、PEの関数範囲・IAT呼出し・静的source locationとの対応を、完全なRust symbolizationや原因確定とは扱わない。

## 2026-10-28 CPU sampleと描画コンパイラの次の比較

同じM673の旧GeneralProfileを実ETLから再解析し、対象processのCPU sample37,539件を抽出した。既存CSVのStackWalkをraw QPC/TID/PIDで完全一致させ、37,536件が結合、3件は未結合のまま除外した。GUI候補TID4344の32長待機区間内には、別threadの結合済みsample37,328件があり、うちmodule範囲外IPの36,942件でstackに`d3d10warp.dll`を含んだ。これは主thread待機中にWindowsのソフトウェア描画経路でCPU処理がある根拠であり、module範囲外のIPを確定したJIT関数や特定shaderとは呼ばない。sample payloadのCountを保存していないため、件数をCPU時間・CPU使用率へ換算しない。

元記録のImageLoadはsystem `D3DCompiler_47.dll`を含み、`dxcompiler.dll`/`dxil.dll`はなかった。公開wgpu-hal30.0.1のAutoがDynamicDXC未発見時にFXCへ進む経路と整合する。待機方式・presentation比較の不成功を踏まえ、次は診断専用の同一EXEでコンパイラだけを比較する。元ZIP/EXE/workerを変更せず、Microsoft公式DXC `v1.9.2609`（2026-09-29公開）の固定asset ID596913261・archive SHA256 `ad31b1fc8443175d204f77a611fdb3ef2ec42759bdc2f1167368de24a4a7e7f1` とx64の2DLLのSHA/PEを検証し、専用一時dirのPATHを対象caseだけに適用・復元する。driver・OS保護・製品設定・200ms条件は変更しない。比較入口は署名付き通常commit `0f08b765`へ統合した。DXCの効果・製品採用・公開成功はまだ未確認。

診断入口の対象検証はcontroller/入力準備/固定asset拒否/receipt整合/traceの合計40件が成功（skip0）。公式assetの実ダウンロード・2DLLのSHA/AMD64・現helperのprepare/verifyを確認した。PowerShell 5 sourceの実構文解析と、receipt hash改変拒否・元の環境変数の復元も成功した。最初のPowerShell実行環境のCoreCLR137は一時書込領域とGC予算の設定後に解消し、本体不具合とは扱わない。新unit fixtureの期待表と記録表の共有によるassertion失敗1件はfixtureを独立化して修復し、製品修復の証明には数えない。独立レビューでP0/P1なし、ambient compiler継承による比較汚染と、成功fixtureからのDLL/receipt改変拒否回帰不足のP2を対応した。これらはローカルの診断入口検証であり、WindowsでのDLLロード・元200ms条件の改善・全OS公開は次の実行で確認する。
