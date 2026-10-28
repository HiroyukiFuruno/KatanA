# Windows起動時の通知取り逃しと描画取得待機

## 正式配布検査の実結果

- source `eb12b3b06b72635bc9636807f27b9b7fcf744ee2` の正式run [40628216485](https://github.com/HiroyukiFuruno/KatanA/actions/runs/40628216485) は終端failure。Linux、macOS arm64/x86_64の配布起動は成功したが、Windowsは元の200ms観測で`2→2`、同じpayloadのServer 2022比較も`8→8`で不合格。Publish/cleanupはskippedであり、公開済みではない。
- 原検査PID9444と、失敗後の独立診断PID6820を区別する。後者のEXEはSHA-256 `eaa01d17268e7b9f6ba1dd1bbee986c94fc87d86399fc4aa94038b54dcf841d8`、127266304 bytes。同runの比較payloadから取得し、診断process記録と一致した。
- 原検査・診断の各ログでは生成後のfilter callbackに無効化3件（125、125、794）があるが、WM_SIZEのforward/suppress記録は0件。登録成功を描画改善の成功へ置換しない。

## 新しい実行ファイルによる待機経路の区別

- 新CSVのImage/Load行10980でPID6820、base `0x7ff613c40000`、SizeOfImage `0x798a000`、実行pathを確認した。PE本体のSizeOfImageとも一致し、旧M234のRVAは流用していない。
- 生ETLのCSwitch/QPCと同じ時点のstackを照合すると、初期thread候補3596に200ms以上の待機32件、対象の復帰stack32件、EventsLost0がある。対象の待機開始stackは欠測であり、復帰stackを待機開始stackと呼ばない。
- app先頭RVAは`0xe875a1`が5件、`0xe7557a`が26件。前者はSurface::configureからDevice::waitへ至るサイズ処理、後者はSurface::get_current_texture_viewからDX12 acquire_textureの追加waitへ至る。残る1件にはapp frameがなく、全32件を同じ原因としない。
- 新PEの両callsiteはIAT `0x1445b37c0`を参照し、同PEのimport tableで`kernel32.dll!WaitForSingleObject`へ解決する。関数の役割は実バイナリのliteral・callsite・分岐と固定版sourceを照合したものであり、PDBの関数名が存在するとは主張しない。
- サイズ処理側はwinitのbuffered-event pop/dispatchとeframeの非zero寸法分岐に対応する。アプリ生成後のcallbackに通知がないため、生成前の通知を取り逃す経路を修正対象とする。ただし全native通知の発生元まで確定したとは扱わない。

## KatanA側の修正

- `eframe::run_native`の前に同じUI threadだけのWH_CALLWNDPROC hookを設置し、タイトルがKatanAで親を持たない所有windowのWM_CREATE時に既存のサイズfilterを登録する。子window・別タイトル・別threadへは登録しない。最初の対象への登録時、またはguard破棄時にhookを解除する。
- guardとcallbackはRcで同じ登録を識別する。古い完了guardが新しいhookを解除せず、native呼出し中にRefCellを借用し続けない。後段のApp生成では既存filterの状態を引き継ぎ、履歴やBoxを置き換えない。
- WindowsのCreateNew GPU設定では、DX12の追加acquire-time waitを既定でDontWaitへ変更する。固定版wgpu-hal 30.0.1の`dx12/mod.rs`1678、1681–1686、1734–1739を照合した。Wait/DontWaitは同じwaitable swapchain flagとSetMaximumFrameLatencyを使用し、後者だけが明示的なhandle待機を省く。描画queue上限の`desired_maximum_frame_latency=1`、shader compiler、adapter/backend/validation flags、既存present modeは変更しない。
- 既存公式環境変数で明示されたWait/DontWait/Noneは尊重し、作成済みGPU instanceは変更しない。新しい環境変数・test port・公開API・安全保護の無効化は追加しない。
- 以前のDontWait単独比較も起動直後の元検査では不合格だった。今回の二経路の区別は、単独設定の過去不合格を成功へ読み替えるものではない。組合せの実Windows検証と元配布検査はこれから行う。

## 対象検証と残件

- 修正sourceは通常署名commit `fb3cabb43f0e00a393d05060e0e695ef9ba37747`へ統合した。通常commit hookは実exit0。push・新HEAD review/CI・公開は別工程として継続する。
- 実Win32 windowを使う新4回帰を追加した。生成時登録・後段の状態引継ぎ、無関係root/child除外、未使用guard破棄、完了guardと次登録の分離を確認する。既存3件の実Win32回帰も維持する。
- 実sourceを参照するMSVC metadata checkと全targetの厳格Clippyは成功。これは新7件の実Windows実行成功ではない。
- macOSでは既存の寸法/DPI/monitor/minimize境界と、実NativeOptionsを使うDX12既定値・明示3値の保持を含む10件が成功した。Windowsの実動作や性能回復へ外挿しない。
- 最終の正式AST30件、fmt-check、対象Markdown lintも成功した。初期の比較用testで外部enumのPartialEq/Copyが未実装だったためコンパイル失敗した記録は保持し、variant照合と移動前の期待値保持へ修正した。assertを削除したりignoreを追加したりしていない。
- 生ログ、同run生成manifest、EXE、ETL、CSV、PE対応表はローカル`tmp/oct28-eb12-*`へ保持。原本や巨大なtraceを公開文書へ埋め込まず、公開後整理でも固有証拠を保全する。
- 通常commit/push、新HEADの全review・必須CI、実Win32回帰、元Windows200ms×10/latency1、正式全OS公開、全5asset/checksum、後処理は継続する。未達を既知制限という理由だけで自動合格にしない。
