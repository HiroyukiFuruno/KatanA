# Tasks: v0.22.34 HTML runtime hardening

## Branch Rule

- KRR: `release/v0.4.4`
- KDV: `release/v0.3.2`
- KatanA: `release/v0.22.34`
- 公開順序: KRR v0.4.4 -> KDV v0.3.2 -> KatanA v0.22.34

## 1. KRR v0.4.4

- [x] 1.1 HTTP/HTTPS stylesheet、script、image を policy 経由で読み込み、mixed content、credential URL、local file escape、unsupported scheme、iframe fetch を拒否する
- [x] 1.2 blocked/failed subresource を document/resource/cause 付きで記録し、主文書描画を継続する
- [x] 1.3 embedded SVG の namespace、attribute case、viewBox、CSS dimensions を保持して browser frame に描画する
- [x] 1.4 unit/integration/AST lint と strict coverage 100% lines / 0 uncovered を除外・閾値緩和なしで通す
- [x] 1.5 v0.4.4 以外を拒否する release guard、package size、publish dry-run を通す
- [x] 1.6 KRR PR の macOS arm64/x64、Linux、Windows、preflight を通し、merge 後に GitHub Release と crates.io の runtime/CLI v0.4.4 を確認する

## 2. KDV v0.3.2

- [x] 2.1 KRR session startup failure 後も worker receiver loop を維持し、valid resize と navigation で session を再生成する
- [x] 2.2 KRR error を operation と complete document origin で包み、worker lifecycle error と区別する
- [x] 2.3 startup recovery、invalid resize、全 browser command/error path の回帰テストを追加する
- [x] 2.4 公開済み KRR v0.4.4 を crates.io dependency として lock し、local path/git source がないことを release contract で証明する
- [x] 2.5 KDV の full check、AST lint、strict coverage、release-check、publish dry-run を通す
- [x] 2.6 KDV v0.3.2 を PR/CI/merge し、GitHub Release と crates.io 公開を確認する

## 3. KatanA v0.22.34 integration

- [x] 3.1 HTML error UI/log に layer、operation、document URL、一次 cause を表示し、後発 WorkerStopped/WorkerPanicked で一次 error を上書きしない
- [x] 3.2 current viewport に一致しない startup/recovery frame を破棄する
- [x] 3.3 fullscreen smooth scroll を x/y pan として扱い、拡大後の zoom と描画寸法を保持する
- [x] 3.4 最大化、右下 controls、fullscreen close を固定黒背景、白 icon、1 px 白 border に統一する
- [x] 3.5 公開済み KDV v0.3.2 / KRR v0.4.4 を registry-only で解決し、workspace と screenshot runner の lockfile checksum を確定する
- [x] 3.6 workspace version、release metadata、CHANGELOG EN/JA、OpenSpec release target を v0.22.34 に同期する
- [x] 3.7 SemVer guard が published v0.22.33 -> v0.22.34 だけを許可し、v0.29.0 と非隣接版を拒否することを確認する

## 4. Headless evidence

- [ ] 4.1 external CSS/JavaScript/image、embedded Mermaid SVG、accordion、button、text/IME、prevented/allowed navigation、fragment、reload、resize の headless acceptance を registry-only binary で通す
- [ ] 4.2 structured primary error と同一 process 内 worker recovery を screenshot と machine assertion で証明する
- [x] 4.3 light theme/白画像上の固定 BG＋border controls を screenshot と pixel assertion で証明する
- [x] 4.4 fullscreen 拡大後の上下左右 scroll で画像寸法が変化しないことを screenshot dimensions/hash と state assertion で証明する
- [ ] 4.5 Chromium/WebView/helper/browser archive が source、dependency、package、process に存在しないことを機械検査する

## 5. Release

- [ ] 5.1 KatanA の format、lint、AST lint、tests、strict coverage、OpenSpec strict validation、release readiness をすべて通す
- [ ] 5.2 self-review 後に関心ごとに signed commit を作成し、`release/v0.22.34` を通常 push する
- [ ] 5.3 PR の必須 CI をすべて通し、承認済み条件を満たした状態で merge する
- [ ] 5.4 GitHub Release v0.22.34、release assets、latest release、withdrawn v0.29.0 不在を live verification する
- [ ] 5.5 merge/release 後に open issue、local branch/worktree、remote branch hygiene を確認する

## 6. User feedback ledger

- [/] 6.1 `HTML browser error: browser worker has stopped` だけで終わらず、原因を UI とログから追跡可能にする
- [/] 6.2 画像を最大化して上下左右 scroll しても元の寸法へ戻さない
- [/] 6.3 light theme または白背景画像でも、最大化と右下 controls を固定 BG と border で視認可能にする
- [/] 6.4 Chromium に依存せず、既存 Rust/V8 CSS/JavaScript rendering 設計を維持する
- [/] 6.5 機械検証後に headless 実機スクリーンショットを提示し、HTML の描画と操作を判断可能にする
