# 配布版の検証記録機構（2026-10-24）

## 本人の決定

- PPTX の抜本改善は KRR の次期対応とし、初回読込の遅延未達を明記して先行公開する。
- 隔離環境で配布版本体の操作・描画・終了を記録する仕組みを追加する。
- 利用規約の承認を代行せず、配布版の公開条件を省略しない。

## 実装範囲

外部 X11 操作と既存 UI の組合せを使用し、製品へ操作専用ポートを追加しない。
通常既定 OFF の完成フレーム記録は、`DEBUG=true` と対になった専用出力先・run ID を指定した場合だけ有効になる。
同一 run ID・PID・連番、文書 URI・generation、host と KDV の資源数を記録する。
取得できない値を推定した 0 や別 run の値で補わない。
診断 ON の結果を通常 OFF の性能、ネイティブ性能、独立品質 95 点の証明として使用しない。

## 検証状態

主査で発見した並列試験 counter の競合は thread-local へ補正し、receipt 8 件、shell 92 件、
厳格 Clippy と fmt が成功した。外部 driver は同一実行の receipt 接続を実装したが、
新機構を含む配布版本体の文書受入は未完了である。
利用規約が未承認の fresh 環境は `blocked_terms` として終了し、合格 manifest を生成しない。

最新の固定 source による Docker run のログは次の通り。

- `tmp/oct24-document-driver-final-fresh-terms.log`: actual 2、利用規約未承認を検知し本体正常終了。
- `tmp/oct24-document-driver-final-legacy.log`: actual 1、撮影画像の Tesseract が既存 15 秒期限を超過。
- `tmp/oct24-driver-final-sources.sha256`: 検査前後で source と保持 archive が同一。

後者の失敗 PNG を同じ observer image・1 CPU・1 GiB で認識すると、通常 1.3196 秒、
`OMP_THREAD_LIMIT=1` で 0.5792 秒、双方終了 0・認識全文同一だった。
元の timeout 自体は再現しておらず、根本原因の確定や製品修復の証明にはしない。
observer の CPU 割当に合わせたスレッド数設定のみを行い、15 秒を含む期限を変更しない。
少 CPU 環境のスレッド設定は [Tesseract 公式 FAQ](https://tesseract-ocr.github.io/tessdoc/FAQ.html) を参照した。

設定後の検査は session 80990 actual 0 で回収した。
`tmp/oct24-document-driver-connected-sources.sha256` の全 script・保持 archive は前後一致。
fresh Terms は actual 2、本体正常終了で未承認を維持した。
legacy は actual 0、900×700 resize・wheel・言語 popup と Escape・WM close の実操作を記録した。
ログは `tmp/oct24-document-driver-connected-terms.log` と `tmp/oct24-document-driver-connected-legacy.log`。
保持 archive は旧本体であり、新 Rust recorder の成功や規約承認後の文書受入へ流用しない。

## 別件の現 CI

公開 HEAD `557079e5588240bbfa2c62f5c7f4777cdf46859f` の Readiness は実成功。
Linux test/build、全 OS lint、CodeQL、供給網も成功した。
Windows は native directory rename の OS error 5 で失敗し、元試験を維持したまま実 watch 構成別の診断を追加中。
Mac headless は新変更のレビューのため PR を Draft に戻した際に workflow が cancelled となり、成功は未確認。
旧 HEAD の成功を新機構へ流用しない。
Rust 機構は署名付き通常 commit `0d76e2a3`、Windows 診断は `d9bf0a8e` に統合した。
正式 AST の初回はコメント形式 1 件で失敗し、既存形式へ修正後に 30 件が成功した。
外部 driver は `6fca4626` へ通常 commit した。
最終 parser/config/receipt 負回帰は 14 件成功、ログは `tmp/oct24-document-driver-contracts.log`。
CLI の import は依存が揃った observer image 内で実行して成功した。
host Python の CLI 試行は PIL 不在で終了 1 であり、実環境成功へ付け替えない。
CLI の責務分離後は対象検査を回収し、先の Docker 成功は整理前 source の結果として保持する。
ログは `tmp/oct24-recording-ast.log`。通常 push と新 HEAD の全 CI は未完了である。
