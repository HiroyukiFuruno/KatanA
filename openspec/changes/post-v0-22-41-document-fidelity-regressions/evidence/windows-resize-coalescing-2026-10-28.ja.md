# Windowsリサイズ通知の重複を抑える候補

## 確認した問題

正式配布run40578480784/attempt1は、元のWindows検査でframe4→4となり不合格だった。同じ配布物を使う任意診断40592186164でもframe5→5を再現した。Linux/macOSの成功や任意診断の採取成功を、元Windows検査の成功には置き換えない。

追加したGPU・DesktopComposition・待機スタックの3profileはexport/start/status/stopに成功し、実イベントを取得した。別診断PID8356の初期スレッド候補8432では、200ms以上のoff-CPU待機を35件観測した。範囲272.386〜322.769ms、中央値279.062ms、ETLのEventsLostは0だった。待機開始stackは欠測し、復帰時stackは35件取得した。

実行ファイルSHA256は`5c7ada7b0bf6d6a79e8a482d52a3fe1202f296a8abed8a3fc6f06e15f556330f`、ImageLoad baseは`0x7FF703860000`。PEのunwind範囲、IAT、RIP相対のsource path・固有文字列と公開sourceを照合し、復帰stackを次の経路へ対応付けた。

`WindowEvent::Resized → Painter::on_window_resized → resize_and_generate_depth_texture_view_and_msaa_view → Surface::configure → Device::maintain(wait_indefinitely) → HAL Device::wait → WaitForSingleObject`

DXGIのPresent API自体は36組のStart/Stopで0.1503〜1.1422msだった。これはGPU処理全体や実表示の時間ではない。フレームのviewport報告は1024×697で一定だったが、WM_SIZE発生元や、各通知の全Win32属性が同一であることまでは確定していない。重複通知の抑制は候補であり、根本原因や性能改善の確定は新Windows実行で確認する。

## KatanA側の変更

- 所有するroot HWNDにだけsubclassを登録し、SIZE種別・非ゼロのclient寸法・DPI・monitorが全て同じWM_SIZEのみ抑制する。
- 最初の通知、実寸法や最大化/復帰の変更、最小化・ゼロ寸法・取得不能は通す。GetClientRectと通知寸法が合わない古い通知も通し、誤抑制しない。
- DPI・画面・設定・style・theme通知で比較状態を破棄する。他の通知は元の処理へ委譲する。
- Cellで同じUI thread内の再入を扱い、委譲中に状態を借用し続けない。WM_NCDESTROYの委譲後に状態を解放する。登録失敗は明示ログへ残し、既存の起動経路を保つ。
- 既存heartbeat設定時だけforward/suppress/invalidateの観測を出す。新しい公開test port・環境変数・OS保護の無効化は追加しない。
- 公開registryのwindows-sys0.61.2をWindows対象へ追加し、両lockへ同じ依存を反映する。KDV0.5.12/KRR0.4.23や既存の予算・Windows200ms×10・latency1は変更しない。

## 検証と未完了

- `just T=native_graphics test-specific`は8件成功。7つの境界回帰と既存の表示設定回帰で、寸法・最大化/復帰・DPI・monitor・最小化/ゼロ・取得不能・状態破棄を検査した。
- 実sourceへのリンクを使う隔離checkerで、公開eframe0.36.2とwindows-sys0.61.2に対しWindows MSVC targetのlib/testsをコンパイル確認した。Windowsでのテスト実行やKatanA全体のクロスビルドの代用ではない。
- 同じWindows target/sourceのClippyも`-D warnings`とwildcard import禁止を維持して成功した。独立レビューの指摘を再確認し、5種の構成変更について登録済み状態の破棄を個別に検査する回帰を追加した。
- sourceは通常署名commit `4ac14b9dc56a116d6fb0cb61ef080fbe15824737`へ統合した。最初のコミット検査で指摘されたコメント形式とテスト補助の数値を規約どおりに直し、通常hook成功と署名Gを確認した。検査や基準の無効化は行っていない。
- 実Win32 windowとSendMessageを使う2件、および登録済み状態の5通知種別を検査する1件を追加した。これらのWindows実行結果、元native検査、通常push、新HEADのレビュー・必須CI・全OS公開は未完了である。
- 独立の機械レビューではP0/P1なし。main全体の`cfg(not(test))`を考慮した再確認を行い、初回のcfg不整合という指摘は撤回された。

## ローカル証跡

- `tmp/oct28-40592186164-gpu-event-inventory.json`、`tmp/oct28-40592186164-owned-waits.json`
- `tmp/oct28-M234-symbols/receipt.json`、`tmp/oct28-40592186164-present-api-durations.json`
- `tmp/oct28-native-resize-final-targeted-tests.log`
- `tmp/oct28-native-resize-windows-source-check-final.log`
- `tmp/oct28-native-resize-windows-final-clippy.log`、`tmp/oct28-native-resize-independent-review.md`

診断CSVの一部は展開SHA256を照合して可逆gzipへ置き換えた。元ZIP/ETLを保持し、移動先と展開hashは`tmp/oct28-derived-csv-lossless-compression.json`へ記録した。元の公開失敗や未達を成功へ読み替えない。
