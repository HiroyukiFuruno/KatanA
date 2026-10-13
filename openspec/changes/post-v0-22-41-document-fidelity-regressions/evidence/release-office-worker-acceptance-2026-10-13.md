# #345 現行 release worker 原本受入

## 対象と実行物

- source HEAD: dd38236e724a0606f888c97a13ea58cbf94c2956。Canvas surface はこの検査時点では未接続。
- registry: KDV 0.5.12 / KRR 0.4.23 / KUC 0.4.1。Issue起票時の旧版指定は履歴であり、最新本人承認の公開registry採用版で再検証した。旧版そのものの再検証とは主張しない。
- worker: target/release/kdv-office-worker、SHA256 92a5481430e506d8d8d11e983e7bcb5417a32de36cbbe5ae97a2f3852f2f4e6b。
- root Cargo.lock SHA256: 5b092e46ed6109d3946298f348cdfcc72fe388e528905eef8d9c562c0950f903。
- build session44038 actualexit0、optimized release 15m02s。raw: tmp/issue345-current-release-worker-build-2026-10-13.log。

## 現行受入結果

- KATANA_DOCUMENT_FIXTURE_DIR に本人提供原本ディレクトリ、KATANA_KDV_OFFICE_WORKER に上記release workerを明示。
- exact test: preview_pane::document_surface::tests::external_document_fixture_dir_reports_typed_first_frame_results。
- session36230 actualexit0、1PASS / 0FAIL / 0ignore、100.10秒。原本6件（PPTX3 / XLSX3）すべて typed first frame 成功。
- 初回表示実測順: 18.919 / 14.643 / 13.039 / 13.187 / 17.968 / 22.295秒。速度改善・非退行の証明ではない。
- raw: tmp/issue345-current-release-worker-acceptance-2026-10-13.log。原本の私的ファイル名・内容は公開コメントへ転載しない。
- 同じ検査binaryでworker環境変数だけを除外した負検査は actualexit101、0.02秒で「external acceptance must name the packaged release office worker」を報告。debug暗黙fallback無し。raw: tmp/issue345-missing-explicit-worker-negative-2026-10-13.log。

## 境界

これは既存harness修復（80266034）の現行release worker再受入である。3OS配布成果物、本人の実機操作、性能基準、#339の配布受入、#338独立描画評価、全release gateの完了を代用しない。timeout・安全閾値・path/git overrideの追加無し。
