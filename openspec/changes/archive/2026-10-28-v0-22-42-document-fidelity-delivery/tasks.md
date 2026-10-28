# Tasks: v0.22.42 3機能の公開後archive

change directory name: `v0-22-42-document-fidelity-delivery`。公開済み3機能の実証拠と限定spec syncを記録する。文書PRのreview/CI/mergeとglobal cleanupはrootの独立タスクで継続し、完了を先取りしない。

## 要件(Verbatim Requirements)

> 軽いpngの切り替えを行うたびにエクスプローラがフリーズしますねｗ
>
> 色だけはすぐ出来ると思うので対応してほしいです。
>
> あとpptxやPDFを表示した際に、キーボードの←→でページングできないのも利便性が低いです。
>
> 今回で言うと実機のユーザー確認は、issueのclose条件に含めない。

### この要件から導出される制約(MUST)

- ローカル画像の読込とUI応答、Markdown分割背景、表示済みPDF/PPTXの左右キー操作だけをこのarchiveへ含める。
- 本人だけの実機確認をClose条件にしない。未実施確認を成功扱いしない。

### 設計判断時の参照義務

- この引用と、元の自動検査・公開条件を維持する。上流の未達をこの3機能へ混ぜない。

## Definition of Ready (DoR)

- [x] 正式runの全OS build/native受入とPublish jobが成功し、current source/tagと公開URLを確認する。workflow全体のconclusionと公開後cleanupは別に記録する。公開前はarchiveしない。

## Branch Rule

本作業はrelease `v0.22.42`の公開後にOpenSpecの3機能deltaを同期しarchiveする。

- **実装統合ブランチ**: `release/v0.22.42` / 通常merge済みPR378。
- **文書後処理ブランチ**: `release/v0.22.42-public-release-evidence`。

- **作業ブランチ**: 新規feature branchを作らず、承認済みのrelease運用で進める。KatanAシリーズの `release/*` 制約を優先する。

## Task 1 — 3機能のcurrent source受入証拠

- [x] 1.1 current source SHAに対して、local PNG async/revision/recovery、Markdown split background、focused PDF/PPTX arrow pagingの既存自動検査とnamed testを対応づけ、実行ログ・fixture identityを保存する。
- [x] 1.2 操作やfeature単位の成功を確認し、PPTX slideshow/preview controlsの15秒未達を本changeに含めずactive umbrellaへ残す。本人実機確認はclose条件にしない。
- [x] 1.3 source delivery根拠は既存PR378とマージ済みsourceを共通利用する。機能/test行ごとに新規source PRを作らない。必要なspec sync/doc変更はまとめて1つのdocumentation deliveryとして扱う。

### Definition of Done (DoD)

- [x] 3機能の現source自動検査と対応する実証拠を保存し、失敗・未実行は未完として残す。
- [x] 実装deliveryは通常merge済みPR378/M8c89の証拠へ対応付ける。spec sync/archiveの単一docs PRはarchive後の独立follow-up（root終了台帳の `oct25_public_assets_and_cleanup`）で追跡し、このchildに重複source PRを追加しない。docs PRの未完を完了扱いしない。

## Task 2 — 公開証跡と限定spec sync

### Definition of Ready (DoR)

- [x] Task 1が完了し、前段の必要なdelivery cycleが終端している。次の作業もKatanA `release/*` 制約の中で行う。

- [x] 2.1 正式release receiptからcurrent source/tag SHA、run/attempt、公開URL、5 assetsの実ファイル名・checksum・取得検証を記録する。
- [x] 2.2 既存main capabilitiesへ3 Requirements/7 Scenariosだけを同期する。`openspec/specs/local-asset-preview/spec.md`の既存Requirementと4 Scenariosは[保存baseline](local-asset-preview-existing-requirement-4-scenarios.baseline)とbyte単位で一致させる。
- [x] 2.3 delta/main差分を併記し、既存requirementの削除・書換えがなく、`preview controls`/`slideshow` deltaを含まないことを確認する。
- [x] 2.4 公開URL/5 assetsの実取得に未回収があればarchiveしない。cleanupは失敗も含めて入力を記録し、rootの独立後処理taskで実行を続ける。

### Definition of Done (DoD)

- [x] 3機能だけのspec sync差分とpublic delivery evidenceを保存し、既存Requirement/4 scenariosが保持されたことを機械比較で確認する。
- [x] 実装deliveryは通常merge済みPR378/M8c89の証拠へ対応付ける。spec sync/archiveの単一docs PRはarchive後の独立follow-up（root終了台帳の `oct25_public_assets_and_cleanup`）で追跡し、このchildに重複source PRを追加しない。docs PRの未完を完了扱いしない。

## KatanA CLI Entry Point

このリポジトリではOpenSpec入口として `./scripts/openspec` を使う。global `openspec` commandの有無で未導入と判断しない。

## 3. Final Verification & Release Work

- [x] 3.1 docs self-review against `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`.
- [x] 3.2 format/lint the combined documentation change; do not edit/change source gates or quality thresholds.
- [x] 3.3 Reuse the already merged source distribution PR378, its current-head review/CI, and merge evidence for all three feature entries. Do not create a separate source PR per requirement, scenario, or named test. If a new docs-only delivery is needed for spec sync, deliver all three additions as one unit under repository rules.
- [x] 3.4 After formal publication, verify the current source/tag/run, public URL, all 5 assets/checksums, retrieval receipts, and required cleanup inputs. If any is missing or unsuccessful, keep this change unarchived.
- [x] 3.5 Pre-archive gate: verify every preceding checkbox in DoR, Task 1, Task 2, their DoD sections, and items 3.1–3.4 is `[x]` with evidence (do not include this item itself or the post-archive follow-up); run `./scripts/openspec validate v0-22-42-document-fidelity-delivery` and `./scripts/openspec status --change v0-22-42-document-fidelity-delivery --json`; confirm the only delta is the selected 3 Requirements/7 Scenarios and existing local-asset Requirement/4 Scenarios are byte-identical.

### Archive operation (after the pre-archive checklist is complete)

上記の全checkboxが証拠付き`[x]`になった後に限り、`./scripts/openspec archive v0-22-42-document-fidelity-delivery` を実行し、日付付きarchive pathを記録する。archive実行自体を事前に`[x]`へせず、完了前にarchiveしない。

archive後に残るglobal cleanupはこのchange/tasks内の未完checkboxに置かず、`tmp/oct27-v42-delivery-draft/oct28-archive-post-archive-followup.md` とrootの終了台帳で実行中・未完として追跡する。未実施cleanupを完了扱いしない。

## 非blocking follow-up

本人の実機確認はarchiveやclose条件にしない。公開後に本人が新たな不具合を見つけた場合は別Issueで追跡する。この項目の完了待ちはしない。
>
> openspec-tasks-templateにはUser Review phaseの必須記載がありますが、このarchive DoDでは本人だけの確認をClose条件にしないという明示指示を優先します。非blocking follow-upへ分離済みです。

## 実証拠索引

- 公開/source/5asset/全CI: `release-evidence.ja.md` とroot `evidence/v0.22.42-public-download-40672472894.json`。
- 新しい検証経路のfresh実取得: root `evidence/v0.22.42-public-frozen-controller-47942653.json`、実行78200 exit0。
- 各機能の現source実テスト: `named-test-mapping.json` / `.md`（8件・各OS実raw行、Unix FIFOはWindows未実行と明記）。
- spec byte保持: `spec-sync-verification.json`、追加3 Requirements/7 Scenarios、既存local1 Requirement/4 Scenarios1222B/SHA944e...一致。
- 独立監査: root作業証拠 `tmp/oct28-scoped-archive-independent-review.md`、P0/P1なし。
- global cleanup failure/未完docs PR: root終了台帳 `oct28_postpublication_cleanup_recovery` / `oct25_public_assets_and_cleanup`。公開済みsourceMと現在の検証コード479を同一と記載しない。
