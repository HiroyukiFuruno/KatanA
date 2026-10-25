# 受入ゲート実装の現HEAD再検証（2026-10-20）

## 対象と実際の結果

HEAD `bd559212f0dba8101e2fa09299e04d97e6e97e75` のtracked clean状態で、既存preflightが使う専用環境 `tmp/release-evidence-python/bin/python` と固定Pillow `12.3.0` を使用した。

- `scripts/release/test-document-fidelity-release-gate.py`: 31件、16.218秒、session54779 actual exit 0。
- `scripts/release/test-document-fidelity-acceptance-evidence.py`: 51件、122.237秒、session80511 actual exit 0。
- raw: `tmp/oct20-bd559-document-release-gate-pinned.log`、`tmp/oct20-bd559-acceptance-evidence-pinned.log`。
- 共有Git設定SHAは検査前後とも `bcf708315eb9ae84c706129108d49862ef47e24a0d13a615f0b3bd58e0fb3c29`。実repositoryの設定を変更していない。

HTML viewport/navigation/geometry、Office missing-elements/geometry、PNG decode、physical footprintの独立予算、Git環境のfixture隔離を含む既存の拒否回帰を検証した。条件、閾値、テスト、fixture、製品コードは変更していない。

## 誤った初回実行との区別

最初にsystem Pythonで直接実行した検査はPillow未導入だった。

- release gate: session11135 actual1、31件中5FAIL、105.851秒。
- acceptance evidence: session45049 actual1、51件中11ERROR、213.617秒。
- raw: `tmp/oct20-bd559-document-release-gate.log`、`tmp/oct20-bd559-acceptance-evidence.log`。

エラー文はrequirementsファイルを案内するが、実コードは `from PIL import Image` のImportErrorで失敗していた。初期の「fixtureへのファイル配置不足」という診断は撤回した。正式環境を確認し、同じsourceを再実行した結果が上記の成功である。失敗を削除・成功扱いせず、今後も既存 `scripts/release/preflight.sh` の専用環境を正本とする。

## 旧修復要求の状態訂正

- HTML P1 `4170924858` とOffice P1 `4170924862` は現HEADの祖先 `f69aeb83` で修復済み。正本はOpenSpec change配下の `evidence/comparison-contract-boundary-2026-10-03.md`。旧未対応行を `[/]` へ訂正した。
- callerの `core.bare` を壊すGit fixture不備は祖先 `b7f41291` で修復済み。正本はrepo rootの `evidence/current-review-regressions-2026-10-02.md`。通常push後の設定不変を含む過去の証跡と、今回の実Git回帰成功・共有設定不変を区別して保持した。

この82件はsynthetic evidenceに対するvalidator回帰であり、原本の表示品質、独立95点、packaged-main、clean-machine、全OS、公開の成功ではない。また、直前の `test_*.py` discovery478件とは別の検査範囲で、同じものを再計上していない。両lockのZIP更新はこの検査終了後に開始しており、この結果を更新後の全依存ゲートへ流用しない。
