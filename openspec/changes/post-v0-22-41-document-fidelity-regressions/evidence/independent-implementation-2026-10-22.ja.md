# 2026-10-22 独立実装の完了範囲

## 判定の訂正

旧「追加の独立実装なし」は撤回した。Windows実測がまだでも、キャッシュ識別、誤再利用拒否、Ready後CI配線は実装できる。上流待ちを理由にその実装を止めない。

## #343 Windows compiled-target cache

- cleanなsource HEAD/tree、両lock、実rustc/cargo版、OS/arch/host/target、profile/feature/selection、Cargo/Rust環境とrunner imageをダイジェストへ束縛した。
- dirty/untracked、未知のCargo/Rust設定、compiler wrapper、識別中のsource変更、版取得の失敗/過大出力はキャッシュ不適格とし、元のbuild/testを実行する。
- Windows test jobだけにexact-keyキャッシュを配線した。prefix restoreはなく、保存対象は`target/debug`と`target/.rustc_info.json`だけ。検査結果やinventoryは保存対象外。
- ヒットでも通常workspace/vendor/HTML trace/cold-font検査を省略しない。Draftの重CI停止、Ready後の元品質条件は変更しない。
- timing診断へcompiled cacheの適格性/key/hitを追加。実子は毎回起動し、失敗exitを維持する。不正metadataでも子の失敗を成功へ置換しない。最終観測scope修正後も14 PASS/7.065秒（`oct22-windows-observer-scope-final.log`）。
- `reuse_authorized=false`、`cold_warm_equivalence=not_evaluated`を維持。Windows実行や短縮を達成したとはしない。

検証ログは作業worktreeの`tmp/`に保持している。

| 検査 | 実結果 | ログ |
| --- | --- | --- |
| 新配線/実子回帰の修正前 | 5 FAIL | `oct22-windows-compiled-red.log` |
| 新配線/実子回帰の修正後 | 5 PASS | `oct22-windows-compiled-green.log` |
| cache identity | 11 PASS、6.168秒 | `oct22-windows-identity-main.log` |
| observer最終 | 14 PASS、1.644秒 | `oct22-windows-observer-final.log` |
| 既存CI契約 | 14 PASS | `oct22-windows-ci-contracts.log` |
| registry/Draft/release flow | 4/5/10 PASS | `oct22-windows-registry-contract.log`等 |

YAML構文の実読は成功。actionlintはこの環境にbinaryがなく実行失敗しており、合格扱いにはしない。Windows用fixtureからPOSIX shell/PATH区切り依存を除去したが、Windows実機の成功はReady後の検証として残る。

## #343 実Rust fixtureのcompile重複削減

- 既存standard coverageと2つのlisting-profile検査で、同一prepared source/manifest pathと同じcallbackを共有する。
- process-localの排他leaseと上限8entryを使用。source/toolchain/環境/callbackの変化、再入leaseは拒否する。
- 全検査、失敗fixture、listing生成、profile生成、report生成は毎回実行する。古い結果・exit・profileは再利用しない。
- private target内の診断/profileだけを掃除し、compile artifactは保持。target/子directory/profileのsymlinkを拒否し、全削除候補を先に検査する。scanは4096entry/深さ32以下。
- default-warmの初回cold assertion、異なるbuild-dir条件、失敗ケース、元selector/timeout/coverage基準は維持する。

同sourceの実Cargo観測は初回0.941秒/fresh=false、次回0.506秒/fresh=true。両方でテスト・listingを実行し、新run IDを生成した。これは小fixtureのcompile削減であり、製品全体の時間短縮やtest省略の根拠ではない。

主査最終検証はguard 7 PASS/4.166秒（`oct22-fixture-cache-main.log`）、実recording/coverage/listing 13 PASS/24.103秒（`oct22-fixture-reuse-main-final.log`）。初版の全release 552 PASS/273.599秒はshared-key/安全修正前の履歴であり、最終source全体の検証へ流用しない。

通常署名commitはWindows配線`d44c01cd0f5fce2b4defe7906beee2e4f2677e3f`、fixture削減`3ebb0712895a34aadfd93cfdf9726a5f5c519d47`、観測scope補正`0efac33dbb105c25b07cae466c68c1b70d81ebbb`。各通常hookの実exitは0。

統合後のrelease Python正本全件を一回実行し、564 PASS/281.072秒・actual exit 0を回収した（`oct22-release-final-integrated.log`）。実行開始は`3ebb0712`で、実行中の製品コード変更はなく、CI observerのscope補正は別の対象14件で検証した。過去545/318.076秒や初版552/273.599秒は実行条件が揃った比較ではないため、総時間短縮の証明に使わない。

## #335 / #338 独立検証

| Issue | 今回実行した独立範囲 | 実結果 | 今回完了に含めない範囲 |
| --- | --- | --- | --- |
| #335 | 実Lefthook fixture、dispatcher rollback、owner/unknown wrapper拒否、cleanup transaction/receipt | 36 PASS/26.221秒、`oct22-335-independent-preflight.log` | 公開OIDに対するproduction activation/live cleanup |
| #338 | 正規Pillow12.3.0環境のacceptance/release gate、retained宣言/hash/寸法、producer unsafe-output拒否 | 51 PASS/65.583秒、31 PASS/8.736秒、manifest/producer PASS | KDV ownerの4軸採点・参照採用・本人受入 |

初回のglobal PythonはPillow不足で11 error。既存の正規venvで再実行して是正し、基準を変更していない。保持済み画像のproducerは`9aa70da5`であり、現在HEADで再撮影した証拠へ置換しない。全4軸のscoreは`not_evaluated`のまま。

## 残るDoD

上流安全版の公開採用、通常push、current review、Ready後required checks、Windows cold/warm、製品全体の同等性/実削減/trusted reuse、全OS packaged受入、本人受入、公開・Issue更新・アーカイブ・cleanupは別の未完項目として保持する。今回の独立実装を全Issue完了やリリース完了と混同しない。
