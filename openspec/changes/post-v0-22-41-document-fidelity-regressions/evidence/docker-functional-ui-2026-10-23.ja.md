# Docker上の配布版UI機能受入（2026-10-23）

## 結論・対象範囲

本人の「Dockerで起動・描画・操作・終了まで検証し、結果回収とコミットまで進める」に従い、配布候補の実ウィンドウをDocker/QEMU上で操作した。最終の正規入口はdriver exit0、アプリ自体も通常終了exit0/OOM=falseである。

確認範囲は新規設定・Terms未同意の初期画面。サイズ変更、Terms本文の上下スクロール、言語popupの開閉、WM経由の通常終了までであり、Terms受諾後のworkspace/Office/HTML/PDF操作・独立95点・native性能・全公開受入の代用ではない。本人のアプリ・profile・Termsを操作/コピー/承認していない。

## 入力と実環境

- 製品は先行clean HEAD `1b7c4b0489449b05e8a14b717b4610f410362e12`から生成した同一tar。以後の差分は検査・文書のみで、製品Rust・manifest・両lockは今回変更していない。
- tar SHA: `82eaa5ff384d1162483590bca73f698710aaeed7f8a5dd39987e370211a27427`。
- 本体SHA: `73cd5832118476bc7df88cfc91dacef51d66f84172388b3be648812b3b8d8e2c`。元architecture verifierが本体/同梱worker双方のELF x86_64を検証した。
- observer image: native ARM `sha256:6a1a9473fb01f666a9b545d75f782c31d4c39f8bdbdb9a41011917dc8c51a372`。Xvfb/Openbox/X11操作・画素/OCR観測だけを担当。
- app runtime image: amd64 `sha256:8f6ff8414fc07730b7fd25c9857d3452f8516e974bdbe953522a0e708805bc66`。`uname -m=x86_64`を検査・記録し、実像はQEMUであることも記録した。
- 同一run専用のX11 socket/evidence volumeとPID namespaceだけを共有。アプリ設定は専用volumeへ新規生成、元tarとrepoはread-only。XvfbはTCPをlistenしない。
- app PID12の正確な`/proc/12/maps`のpath、cmdlineの完全な引数、本体SHAを入力tarと照合。ネイティブ用の既存identity guardを改変・skipして合格にしたものではない。

## 最終実行の結果

正規入口: `scripts/release/check-docker-ui-functional.sh`、実行session14884 actual0。

| 項目 | 実結果 |
| --- | --- |
| 起動 | 実ウィンドウ、完了frame、guest path/maps/argv/SHAを確認 |
| 描画 | 初期2観測のblankを保持し、次の観測で画素分散21.834とOCR `Terms of Use`を確認。記録SHAを比較基準には使わない |
| サイズ変更 | X11応答が900×700、再描画とTerms文字を確認 |
| スクロール | 上端の`Thank you for using KatanA`を観測後、wheel入力し、下端の`Anonymous crash reports`/`By clicking`を観測 |
| 言語popup | 実クリック後の`Deutsch`表示、Escape後のpopup消失とTerms表示を別々に観測 |
| 正常終了 | WM close要求後にウィンドウ消失、`docker wait`が0。最後のframe117。TERMによるcleanupを正常終了とは扱わない |
| Terms | 設定`terms_accepted_version=null`、Accept/Decline非操作 |
| 後処理 | driver0・強制終了marker無し。証跡export後、所有する停止済みcontainer2件と固有volume2件だけを削除、再inspectでも不存在を確認 |

最終run: `tmp/docker-ui-functional.altw5I/`。機械結果は`evidence/functional.vpvz0enl/result.json`、PNG/OCR、config、heartbeat、application log、両exit、image/container metadata、削除volume名、archive SHAを保存した。`native_performance=false`を明記する。PNGの目視だけではなく、X11状態・画素の非blank・OCR内容・プロセス状態/実終了コードを検査した。

## 失敗の回収とKatanA側の修復

- 初回observerに`xdpyinfo`がなくserverがexit1、同じPID namespaceのappは137/OOM=falseとなった。環境欠落を実ツール一覧で特定し、observerへ`x11-utils`を追加した。製品クラッシュとは扱わない。
- フレーム完了通知がGPUの実表示より先だったため、初期の観測実装がblankで即拒否した。実表示の画素と要求文字列が揃うまで条件待ちするよう検査を修復した。固定sleepだけで合格にしていない。
- OCRが`servers`を誤認した初回scroll判定は失敗として保持した。実画面の下端にある別の明確な2文言を応答条件とし、上端→入力→下端の遷移を検証した。曖昧一致や成功の手書きではない。
- 本人が許可したDocker機能検証用の独立入口とobserver Dockerfileを追加。既存native smokeの0.2秒観測・path/hash検査・RSS/font予算は変更していない。本入口の90秒条件待ちはQEMU上の機能観測期限であり、native性能基準の延長ではない。
- 実Linux子プロセスの4回帰は全成功（0.122秒の先行実行、最終ファイル名でも再実行0）。正しいmapped file、誤SHA、同prefix別path、偽argvをモック無しで検査。Linuxの`/proc`を用いる明示Docker統合検査であり、通常macOSのstdlib UTへPillow等を追加していない。
- ARM imageをapp runtimeに誤指定する実negative実行は、container起動前にexit1で拒否した。独立レビューのarchitecture証跡/所有資源整理の指摘も修復・最終E2Eへ含めた。必須P0/P1なし。

## 再実行と物理証跡

observerは`scripts/release/docker-ui-observer.Dockerfile`をホストnative向けにbuildする。画像ツールはobserverだけに必要であり、製品依存へ追加しない。

```bash
bash scripts/release/check-docker-ui-functional.sh \
  tmp/oct23-linux-package-retained/KatanA-linux-x86_64.tar.gz \
  katana-docker-ui-observer:20261023-final \
  katana-packaged-linux-runtime-x86_64:20261023
```

- `tmp/oct23-docker-ui-final.log`、`tmp/docker-ui-functional.altw5I/`が最終正本。先行成功`tmp/oct23-docker-ui-canonical.log`と`tmp/docker-ui-functional.xwBb8K/`とは区分する。
- `tmp/oct23-docker-ui-observer-build{,-v2,-final}.log`、`tmp/oct23-docker-ui-functional{,-v2,-v2-retry,-v2-retry2}.log`、`tmp/oct23-docker-ui-failed-scroll-ocr/`に失敗・修復履歴を保持する。
- `tmp/oct23-docker-ui-provenance-final-entry.log`、`tmp/oct23-docker-ui-wrong-architecture-negative.log`、`tmp/oct23-docker-ui-final-inputs.sha256`。最終runtime入口のsource/両lock/tarの検査前後一致を主査確認した。

Docker機能確認をnative host未提供のため停止する状態は解消した。上流安全公開版、通常push/current review、Ready後必須CI、全配布/本人最終受入、公開・後処理は引き続き別の未完条件である。
