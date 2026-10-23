# Linux配布候補の起動検査・原因切り分け（2026-10-23）

## 結論

今回の`launched process image did not match packaged executable`は、ARMホスト上のQEMU user-mode実行で、Linuxが実行像としてゲストのKatanAではなくエミュレータを返したためである。同一tarから起動した実KatanAのPIDと`/proc/<pid>/exe`を調べ、推測から実測へ進めた。製品のnative Linux起動障害を証明したものではなく、native起動成功とも扱わない。

KatanA所有の修復は、起動検査の同一性FAIL時に期待する実行パス・SHAと実プロセスツリーのimageを残すこと。合格条件、path/hashの比較、期限、RSS/font予算、同梱物、製品Rust、依存版、Draft CI方針は変更していない。

## 同じ配布物の実測

- 製品HEAD `1b7c4b0489449b05e8a14b717b4610f410362e12`で生成済みのtarを使用。tar SHAは`82eaa5ff384d1162483590bca73f698710aaeed7f8a5dd39987e370211a27427`、本体SHAは`73cd5832118476bc7df88cfc91dacef51d66f84172388b3be648812b3b8d8e2c`。再ビルドや内容変更はしていない。
- 既存の正式`inspect-release-asset.sh`をローカル候補ディレクトリ指定で実行した。トップレベル`KatanA`/`kdv-office-worker`、両ELF x86_64、0755を確認。exit0だが、公開用`checksums.txt`不在の警告は保持する。公開checksum成功の証拠ではない。
- amd64 runtime image `8f6ff8414fc07730b7fd25c9857d3452f8516e974bdbe953522a0e708805bc66`の実プロセスを観測した。本体PID69のcmdlineに抽出したKatanAがあり、`/proc/69/exe`は`/usr/bin/qemu-x86_64`、実行像SHAは`bb39691198d944c52c142c9cd4cb7a5b0cdcb7146a3b04c60c4b34ddee5aebe9`だった。timeout、xvfb-run、Xvfbも同じエミュレータ像だった。
- 元の`packaged_find_matching_pid`はexit1。診断用の描画観測もframe1から3へ進んだ後、元の短間隔検査で停止を検出しexit1となった。初期描画・font記録だけを継続操作性や性能合格にしない。
- 修正後の元smokeを同じtarで1CPU/2GiB、さらに2CPU/2GiBで実行し、両方exit1/OOM=false。2CPUのログでは期待本体パス/SHAと、launcher/子4PIDの実像`/usr/bin/qemu-x86_64`を回収できた。CPU割当の違いを記録し、期限延長やidentity guard緩和はしていない。

正式検査の修正後ログ抜粋（原文）:

```text
FAIL: launched process image did not match packaged executable (linux-x86_64)
Expected process image: executable=/app/tmp/oct23-linux-cross-extracted.9mwroz/KatanA; sha256=73cd5832118476bc7df88cfc91dacef51d66f84172388b3be648812b3b8d8e2c
Observed process image: pid=80; image=/usr/bin/qemu-x86_64
Observed process image: pid=86; image=/usr/bin/qemu-x86_64
Observed process image: pid=117; image=/usr/bin/qemu-x86_64
Observed process image: pid=126; image=/usr/bin/qemu-x86_64
```

## 修復と回帰検証

- `packaged-process-identity.sh`へ実PIDのimage診断を追加した。空PIDを拒否し、深さ8で診断再帰を制限する。cmdlineを本人認証の代用にせず、既存の`/proc/exe`/`lsof`観測だけを使用する。
- `smoke-launch-packaged-app.sh`の同一性FAIL3箇所からのみ呼ぶ。成功経路・合否条件は不変。終了競合では`image=unavailable`を記録できるが、元FAILを成功へ変えない。
- 先に実子プロセスのimage出力・空PID拒否の回帰を追加し、旧実装で関数不在のexit127を回収した。修復後は、既存の誤path・誤SHA・終了PID拒否を含めて成功した。モックプロセスや差し替えた検証関数は使用していない。
- macOSで元`test-packaged-startup-contract.sh`はexit0。Linux ARMの独立containerでも、同じ正式契約を実行してexit0。Linux x64のKatanA起動合格ではなく、検査コードと実プロセス観測の回帰証明である。
- ARM対照試験の初回はimageに`pgrep`/`ps`がなくexit1だった。実ツール一覧で欠落を特定し、所有containerだけへ`procps`を導入して正式契約を成功させた。別の直接nativeプロセス対照では`/proc/exe=/usr/bin/sleep`を確認し、正path/hashを受け入れ、誤path/hashを拒否した。
- Bash構文・diff-check成功。ShellCheckはsource追跡指定で既存の変更外trapにSC2329（info）が残る。warning以上は0、抑制コメントやルール変更は追加していない。対象の独立レビューで必須P0/P1なし。

## 物理証跡と残件

- 実測: `tmp/oct23-linux-candidate-inspector.log`（正式inspect出力を原文保持）、`tmp/oct23-linux-startup-diagnose.log`、`tmp/oct23-linux-startup-diagnostic-heartbeat.txt`。
- 元smoke再実行: `tmp/oct23-linux-startup-diagnosed-smoke.log`、`tmp/oct23-linux-startup-diagnosed-smoke-two-cpu.log`、`tmp/oct23-linux-startup-diagnosed-retained/`。成功に変更せず保持する。
- 回帰: `tmp/oct23-process-image-diagnostic-red.log`、`tmp/oct23-process-image-diagnostic-contract.log`、`tmp/oct23-linux-native-identity-control.log`（procps欠落時の失敗）、`tmp/oct23-linux-native-identity-tools.log`、`tmp/oct23-linux-native-identity-direct.log`、`tmp/oct23-linux-native-startup-contract.log`。
- 本人profile/Terms/アプリは操作せず、所有containerは終了済み。新worktree、上流実装、Draft重CI、通常hook迂回は行っていない。
- Linux x86_64 native・Windows/MSI・Intel実機・clean-machine・配布main全受入は未完。提供済みのnative経路は通常push/current review/Ready後CIであり、今回の互換実行をその代用にしない。KDV #69安全公開版待ち・全5asset公開と公開後処理も維持する。
