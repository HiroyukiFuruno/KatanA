# 配布版の検証記録機構（2026-10-24）

## 本人の決定

- PPTX の抜本改善は KRR の次期対応とし、初回読込の遅延未達を明記して先行公開する。
- 隔離環境で配布版本体の操作・描画・終了を記録する仕組みを追加する。
- 最新の本人直接承認で、今回fresh隔離QAに限る自動同意を許可。本人用アプリ・設定・prepared profileには適用せず、公開条件や最終本人受入を省略しない。

## 最新のmacOS検証への移行

本人はDocker制約を了承し、保護を無効化せずmacOS隔離配布版で文書操作・描画・終了を検証するよう指示した。承認済み[更新目標](release-goal-2026-10-24.ja.md)を物理正本へ固定したが、Codex画面の目標編集は安全上拒否され、画面上の旧目標は未変更と明示している。

新fe2ae190のLinux archive生成・構成検査は実終了0。fresh規約の実クリック・42保存・Explorer exact filename double-click・source intake/worker handoffは確認したが、QEMUのLandlock ABIがENOSYS38で、KDVがfail-closedとして文書描画を拒否し、全体は実終了1。ARM native observerはABI4を返した。保護を無効化して合格扱いにしない。default同意なしの新配布検査はdriver2/app正常終了0を確認した。

外部メモリ観測は865b1f6b、HTML受理済み状態はf0f654db、明示profileのcache/lint保存先・Homebrew副作用分離はc7f3a2b7へ通常署名commit済み。対象の単体検査・独立レビューは各証跡に記録した。旧通常commitのAST失敗や中断130も保持し、成功へ付替えていない。

Dockerの単語部分一致によるTerms誤検出は共有heading/version/controls判定へ修復したが、OCR座標を同一modal領域へ結合するP2は未完として残す。macOSは外部UI操作で別途検証し、Linux診断結果を成功へ代用しない。

最新macOS配布物の生成・署名・隔離profileでの実文書受入は次工程である。本人の通常profileへcacheが漏れないよう、実行前に[profile境界](isolated-profile-boundary-2026-10-24.ja.md)を修復した。

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
ログは `tmp/oct24-recording-ast.log`。

通常 push は session 5962 actual 0、`tmp/oct24-packaged-recording-normal-push.log` で回収した。
local/origin/PR の `0d057ec6d9b4cf9d75c4207f726358ac13c11d16` 一致を確認し、
Issue #338 の [状態コメント](https://github.com/HiroyukiFuruno/KatanA/issues/338#issuecomment-6382401268) を確定更新した。
Draft の CI 40081054726 と Readiness 40081054722 は skip であり、必須検査成功とは扱わない。

## 承認範囲と公開判定の整合

`katana-host` の PPTX 初回表示が 15000 ms を超える場合だけ、固定 8 field の
`first_frame_disposition` を要求する。生の実測値、未達、本人承認 token、今回の版を保持する。
strict、DOCX/XLSX、正常終了、RSS、資源 cycle、描画条件は変更しない。
記録がない、測定値が異なる、他形式への流用、未知 field、NaN/inf/bool は拒否する。
閾値の緩和でも性能合格の主張でもなく、承認された既知制限の明示だけである。

旧実装に対する RED は `tmp/oct24-pptx-first-frame-contract-red.log`、終了 1。
対象契約 54 件は `tmp/oct24-pptx-first-frame-full-final.log`（78.444 秒）、追加境界 3 件は
`tmp/oct24-pptx-first-frame-focused-final.log`、差分 10 件は
`tmp/oct24-pptx-first-frame-divergence.log` で終了 0 を回収した。
これらは契約試験であり、実 manifest や実配布版の成功へ付け替えない。

DOCX/PDF の CLI 接続は通常署名 commit `fcfd7034`、指定できる形式と記録処理の
15 件は `tmp/oct24-document-driver-all-formats-contracts.log` で成功した。
実 observer image 内の CLI も 4 形式を表示して終了 0。
新 HEAD のレビュー、必須 CI、新配布版の受入、公開は未完了である。
