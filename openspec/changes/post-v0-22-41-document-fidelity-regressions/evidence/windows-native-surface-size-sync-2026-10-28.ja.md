# Windowsの初期surface再設定を描画時の実寸へ揃える

## 回収した結果

PR377のH `ea5575cf06e029973bb7eff95fc6edf975f6843c` は全必須CI・Codexレビューを完了し、署名valid・tree同一のM `991c2b41fd0fa0f12deb8c797add5b3f7fe3dc48` へ通常mergeした。Mのpush CI40648368440も成功した。

正式direct公開40648617381はWindows元smokeのframe3→3で不合格になった。Linux・macOS両CPUと同一Windows配布物のServer2022比較は成功したが、Publish/cleanupはskipped。元Windows環境の不合格を比較環境の成功へ付け替えない。全5公開物は未確認。

元logは `tmp/oct28-991c-direct-windows-package-failure.log`。当該EXEは127285248 bytes、SHA256 `86c5fad75b2b7f276d9b13cb24076cd649c5b354699f7e332ef327b85dd19c1c`。原smoke PID3148と診断PID7384を区別した。

## 待機とソースの対応

- 新しいEXE・ImageLoad・PE `.pdata`・IATを照合。診断の初期thread候補8436に200ms以上の待機5件、復帰stack5件、lost0を回収した。初めの4件は349.527/314.034/297.909/306.264msで、同じapp RVA `0xe89be1` から `kernel32.dll!WaitForSingleObject` へ到達する。新binaryの文字列/制御フローはDevice::wait、Device::maintain、Surface::configure、Painter resize、eframe/winit buffered dispatchに対応する。残る1件は別stackであり、この4件と同一原因とはしない。
- 診断logではWM_SIZEのforward4件が07:54:48.063–48.152、最初のUI frameが48.417。viewportはその時点で最終1024×697。forwardされたサイズは1024×768→1024×697→1024×768→1024×697。生成時hookの登録とduplicate1件の抑制も確認した。
- locked eframe0.36.2は初期surfaceをwindow.inner_sizeで作成した後、Resized payloadをegui-wgpuへ渡す。egui-wgpu0.36.2はpayloadごとにsurface寸法を更新してconfigureする。古い初期通知を描画後に処理すると再設定が必要以上に続く経路がある。
- winit0.30.13のWM_SIZE handlerは最大化cacheも更新するため、KatanA側で初期WM_SIZEを単純に捨てる案は採用しない。既存のnative通知と状態更新を維持する。

生ETL/CSV、診断log、PE/import対応は `tmp/oct28-991c-windows-startup-diagnostics/`、`tmp/oct28-991c-owned-waits.json`、`tmp/oct28-991c-symbols/current-stack-api-literals.json`。PDBによるsymbol解決や全GPU待機原因の確定を主張しない。

## 修正候補

公開済みegui-wgpu0.36.2のcrate archiveを既存lock checksumと照合して `vendor/egui-wgpu` へ取り込み、root/screenshot両manifest・lockへ同じpatchを適用した。crate SHA256は `5cef1f6d1bba40b09a4d0d62cb822b81030922cfbf65c5373398c3a3caafc5bd`、元commitは `49682f8baa058bf49e011035cfbd6e825f88a5ef`。MIT/Apache licenseはその公式commitから取得した。不要な元standalone lockは証跡側へ保持し、root/screenshotの既存lockを正本にする。

Windowsのみ、Resized通知時の即時configureを描画時へ移し、実windowの現在client寸法を読む。正寸法が現在surfaceと異なる場合だけ、既存のresize/configure処理を行う。同じ寸法とゼロ寸法では新しいconfigureをしない。ゼロ寸法を理由にpaintから新たに早期returnせず、既存texture upload・SurfaceLost/Outdated・runtime config処理を維持する。他OSのイベント時resize経路は変更しない。

既存のGPU queue上限1、DontWait明示設定の扱い、backend/compiler/validation/保護機能、元200ms×10のsmoke条件は変更しない。KDV/KRRは既存公開registry pinを維持し、別repoを実装しない。

## 対象検証と残件

- 元の「毎回再設定する」判定を抽出した状態で、同寸法を再設定しない回帰が実FAIL、他2件PASS。修正後は同寸法・実寸変更・ゼロ寸法の3件がPASS。`tmp/oct28-native-surface-size-red.log` / `tmp/oct28-native-surface-size-green.log`。
- production寸法判定moduleを直接検査する `just test-wgpu-native-surface-size` を通常testと全3OS CIへ接続。GPUを模擬するITは追加しない。
- root/screenshot `cargo metadata --locked --offline` は成功し、font依存の意図しない再選択を戻して、両lockはegui-wgpuのsource切替だけに限定した。
- 既存 `just type-check` と実sourceを使うWindows MSVC all-targets Clippyは成功。これはWindowsの実行成功ではない。独立差分確認はP0/P1なし。

sourceは署名Gの通常commit `b2a14ce4fdf574412d6f12aafeed3a9a00eab7f3` へ統合した。通常push、新HEADの全review・必須CI、Windows元配布smoke、全OS公開、全5assets/checksum、後処理は未完。候補の型検査・純粋判定testを、性能修復や公開成功へ置き換えない。
