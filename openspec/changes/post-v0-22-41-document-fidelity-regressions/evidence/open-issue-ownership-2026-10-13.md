# Open Issue 所有責務と残作業（2026-10-13）

GitHub live `gh issue list --state open` は9件。下記はIssue全体の完了判定であり、一部実装・テスト成功をCloseへ置換しない。

| Issue | 現状態 | KatanA側の残作業 |
| --- | --- | --- |
| #258 Canvas | source通常commit済み | core/host/surface/read-only/非同期/i18nは正式履歴へ統合。Canvas57・既存fullscreen23・AST23/strict lint成功。公開push/fullgate・review/残受入・Issue整理は未完。 |
| #312 PDF theme | 修正・実出力検証source通常commit済み | custom/builtin明暗4テーマPDFの独立設定token・色/コントラストと負検査成功。公開push/current fullgate・Issue整理が残る。KDV文字重なりとは別。 |
| #335 hooks | 一部実装・公開branchへpush済み | commit参照syntax15件は成功。live Issue照合、下流migration/manifest/lock証跡、dispatcher順序、公開後safe Git cleanupは未対応。 |
| #336 KLE adoption | KLE公開版待ち | KLE v0.1.0 public tag/registry未確認。公開後の固定registry採用と実editor host入力→最終効果検証も未完。KUC公開だけでは完了しない。 |
| #338 reference crop | 未完 | 正規producerの基準再生成、独立意味/操作/性能検証、公開/配布graphとの結び付けが残る。単にKRR待ちとは扱わない。 |
| #339 downstream acceptance | 依存採用済み、受入未完 | 現固定KDV0.5.12/KRR0.4.23を維持し、全OS配布アプリ・原本・正常終了・元メモリ基準の受入を完了する。worker単体は代用しない。 |
| #340 data URI | 修正・限定検査成功 | parserとhost previewの正確なfilterは成功。元sample全体のexport/配布表示が未検証。原fixtureの不正SVG namespaceを改変して成功扱いしない。 |
| #343 validation optimization | 一部実装・公開branchへpush済み | registry-only Windows cache済み。test/coverage inventory、証明付きreuse/再実行、安全拒否、cold/warmと総時間比較が未完。Draft非実行/Ready全ゲート方針維持。 |
| #345 release worker harness | 実装・現行原本受入成功、証跡通常commit済み | release workerでPPTX3/XLSX3全6 typed first frame、未指定worker拒否成功。証跡公開push後にIssue整理する。ロード13–22秒を改善済みとはしない。 |

## 公開状態

- PR346はDraft、公開HEAD19e5e1fe7fa61d599cf91432328931b9345e9734。
- 最新local sourceはCanvas dec02e313f7b9fda0e1b19604f8f1cf6d10e7221/PDF検査 b1a5de11d4f14d7bb7722ef347f1253e8b094bc8/345証跡 b3ac38c07f32a636ada179022b84559158ce7fdfへ通常commit済み。公開push/current fullgateは未完。
- KatanA v0.22.42 GitHub Releaseはlive `release not found`。公開完了とはしない。
- primary master clean、stash0、既存2worktreeのみ。新worktree・上流実装・通常stash・hook迂回無し。

上流待ちは独立作業の停止理由ではない。#336の公開待ちと、その他の未対応/検証/正式統合/配布受入を別々に追跡する。
