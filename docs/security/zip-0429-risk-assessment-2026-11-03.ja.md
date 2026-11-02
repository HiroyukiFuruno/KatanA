# ZIP 0429の限定評価と期限付き管理

## 判断と範囲

既存のZIP利用経路評価と期限付き先行公開の判断に合わせ、RUSTSEC-2026-0429だけを現行の公開registry graphに限定して管理する。旧0367/0368/0369の理由・対象経路・期限を変更せず、新IDの評価をこの記録へ分離する。全ての新規脆弱性を免除する判断ではない。

期限は既存と同じ2026-11-07 00:00 UTC（日本時間09:00）。この具体的日付は担当が以前設定した運用上の再評価期限であり、本人による日付の明示承認や上流公開予定ではない。今回その期限を延長しない。公開registryの修正版採用・利用経路の変化・新たな悪用証拠があれば期限前でも再評価し、自動延長しない。追跡は[KatanA #354](https://github.com/HiroyukiFuruno/KatanA/issues/354)と[KDV #69](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/69)。

## 対象問題

[公式RustSec](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/zip/RUSTSEC-2026-0429.md)はCVE-2026-115909 / [GHSA-q7v9-x93c-xqv3](https://github.com/zip-rs/zip2/security/advisories/GHSA-q7v9-x93c-xqv3)に対応する。advisoryの日付は2026-10-22、RustSec DBへの追加は2026-11-02 09:50:37 UTC（be2abb0371ae57e4a877e112d598a0d9bfcdb347）。修正版範囲はZIP >=9.0.0。

ZipArchive::extractのsymlink処理が攻撃者の宣言長からバッファを確保し、確保拒否でprocessをabortする問題。panic捕捉、worker thread、timeoutだけでは一般的な保護にならない。有害PoCは実行していない。

RustSecはunaffected範囲を持たずZIP0.6.6も検出するが、GHSAはaffected下限を1.3.0とする。0.6.6のextract実装には当該symlink確保分岐がない。ローカルで公式DBを改変せず、この定義差も記録する。2.6.1/8.6.0には当該分岐があり、0.6.6の訂正だけで問題全体がなくなるとは扱わない。

## 現行の有限利用経路

root・screenshot・paint-metricsの3manifest、macOS arm64/x86_64・Windows MSVC・Linux GNUの4targetについて、locked/offline/all-featuresのmetadata12件を実取得し全終了0。root/screenshotの8graphは同じZIP署名、既存6snapshotとも一致。paintの4graphにZIPはない。

| ZIP | 直接consumer |
| --- | --- |
| 0.6.6 | ironcalc0.8.3、office2pdf0.8.1 |
| 2.6.1 | umya-spreadsheet2.3.3 |
| 8.6.0 | docx-rs0.4.22、katana-document-viewer0.5.12 |
| 9.2.2 | local katana-core0.22.47 |

旧ZIPの5consumerの公開registry source、src内2037ファイル（testを含む過大集合）を調べ、ZipArchiveのextract/extract_internal/extract_unwrapped_root_dir、ZIP stream展開、該当reexportの使用は見つからなかった。実Office経路はcentral entryの検査とXML/asset読取り。汎用extractの一致はPDF画像やXML callbackでありZIP展開ではなかった。KatanA updaterはZIP9.2.2の独自entry loopでリンクを拒否し、font safetyも9.2.2を使う。

これは有限graphと明示APIの静的評価であり、全プログラム・将来構成・全動的経路の非到達や安全性を証明していない。metadata成功は供給網検査成功でも配布版受入成功でもない。

## 更新と解除

公開KDV0.5.12のZIP/ironcalc/office2pdfはnonoptionalでexact制約を含む。公開docx-rs0.4.22もZIP8.6を必要とする。各旧ZIP系列の最新互換版は現在のlock版と同じで、rootのZIP9更新やfeature解除だけでは旧版を除去できない。未公開path/git上書きや無断の別repo変更は行わず、上流の修正版がregistry公開されたら実graphと全品質を検証して採用・例外解除する。

## 検査条件

正式入口は固定版・親子関係・KDV経由・公開registry source・workspace到達性・絶対期限を検査後に限り一時cargo-deny設定へ0429を追加する。恒久deny.tomlや他のadvisory/ライセンス/品質条件は緩和せず、cargo-denyの失敗を伝播する。未知ID・版・別親・KDV迂回・非registry・孤立・期限切れは拒否する。全3manifest供給網、対象回帰、独立レビュー、正式push/全CI/公開物検証を別に回収する。配布版の検証や公開完了はこの評価記録だけから主張しない。

既存の共通入口は、ZIPを含まないpaintの一時設定にも未使用のignore IDを生成する。paintに旧ZIP依存があるという主張や、新たな依存経路の許可ではない。全manifestでmetadata検証と既存cargo-denyの全項目を実行する条件を維持する。

根拠原本はtmp/pr405-resolution/rustsec0429-independent-review.md、rustsec0429-graph-assessment.json、rustsec0429-api-assessment.json、registry-resume-v47.json。公開記録には非公開原本の内容・pathや個人設定を含めない。
