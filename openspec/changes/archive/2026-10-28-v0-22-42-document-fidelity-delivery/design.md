# Design: v0.22.42 3機能の公開証跡とarchive

## Context

現active `post-v0-22-41-document-fidelity-regressions` はより広いumbrellaであり、将来・未完条件を含む。今回のarchive候補は、公開後に範囲を分離できる3機能だけとする。配布元M `8c89a1544e9ac693d5ba388792a843baa55c9363`、PR378 merge済み。CI実checkout2aa/H a658/Mは同一tree、全CI/Readiness/current review完了。正式direct40672472894の全OS build/native/Publishと全5実取得が成功。cleanup installerだけ失敗しrun全体failureで、rootの後処理taskが修復を継続する。

## Goals / Non-Goals

**Goals:** 完了した3機能の自動検査と公開ソースを対応付け、既存specを保持した追記と限定archiveを行う。

**Non-Goals:** 製品の再実装、上流レンダリング全体の95点達成の主張、PPTX初回15秒未達の解消、active umbrellaの一括archiveは行わない。

## Decisions

### Capability境界

既存main capabilitiesへ追加Requirementだけを同期する。新capabilityは作らない。local assetは既存requirementを置換せず追加し、本文と4 scenarioの既存blockをbyte-for-byte保持する。現checkoutにユーザー指定の `openspec/specs/local-asset-loading/spec.md` はなく、対応する実在specは `openspec/specs/local-asset-preview/spec.md`。実在pathに同期することを親が確認する。

### 3機能のdelta

`specs/` 内の3 delta specは、旧draftから対応するADDED Requirementだけをそのまま抽出する。local PNG async/revision/recoveryは3 scenarios、Markdown split backgroundは2、focused PDF/PPTX arrow pagingは2、合計3 Requirements/7 scenarios。

`preview controlsはactive document formatと読込状態に従う` とその全Scenario、特にPPTX slideshow readinessは含めない。通常PPTX初回15秒の失敗はactive umbrellaの未達として残す。

### Evidence

source/test referenceは実行証明ではない。final source SHAに紐づく自動検査、current required CI/review、実際の正式CD終端、公開tag/sourceと5 assets/checksum/取得記録を別々に保存する。公開証跡はrelease-evidence.ja.mdとrootの実取得receiptへ対応付け、cleanup failureと未完のroot後処理を区別する。

## Risks / Limits

- 3機能の自動検査やWindowsの実Win32全7件/native-size判定3件のPASSは公開完了、5 asset取得、他targetでの全操作packaged証明の代わりではない。
- async image contractは既存の協調checkpointにおける古い結果の破棄であり、任意OS-blocked read/codecの強制中断を保証しない。
- OpenSpec sync/archiveは正式公開後にdeltaを確認して実施する。active umbrellaはarchiveへ移動しない。

## Migration Plan

正式公開と全5assetの実取得後、同じソースの後処理branchで3つのmain specへ追記し、baseline一致・正式validate/statusを確認してchildだけをarchiveする。docsを単一PRへ集約し、通常レビュー/CI/merge後に安全なcleanupを行う。製品ソースの変更や再タグ付けはしない。

## Open Questions

この限定archiveの対象範囲に未決選択はない。公開・実取得が未完の場合はarchiveを実行せず、root作業台帳で実行を継続する。
