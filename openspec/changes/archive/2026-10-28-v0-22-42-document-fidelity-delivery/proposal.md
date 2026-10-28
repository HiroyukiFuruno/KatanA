## Why

v0.22.42の実装済み機能を、公開結果と自動検査の根拠に対応付けて正式specへ同期する。active umbrellaは将来・未完条件を含むため、完了した3機能だけを分離してarchiveする。

## What Changes

- local PNGの非同期読込・revision破棄・欠落後復旧を、既存local asset specへ追加する。
- Markdown分割表示の背景色をExplorerとそろえる要件を追加する。
- 表示済みでfocusを持つPDF/PPTXの左右キーページング要件を追加する。
- 同期は3 Requirements/7 Scenariosに限定し、既存のlocal asset Requirement/4 Scenariosをbyte単位で維持する。
- preview controls/slideshow全体とPPTX初回15秒未達は、active umbrellaへ残す。本人確認をClose条件にせず、未実施確認を成功と記録しない。

## Capabilities

### New Capabilities

なし。

### Modified Capabilities

- `local-asset-preview`: local PNG async/revision/recovery 1 Requirement/3 Scenariosを追加。
- `workspace-shell`: Markdown split background 1 Requirement/2 Scenariosを追加。
- `multi-format-document-preview`: focused PDF/PPTX arrow paging 1 Requirement/2 Scenariosを追加。

## Impact

実装は既存の通常merge済みソースを利用し、新たな製品コード・API・依存関係の変更は行わない。公開証跡、既存main specへの追記、限定childのarchiveを単一の後処理docs deliveryへまとめる。umbrellaや上流の未完条件を完了扱いにしない。

## Delivery status

PR378は通常merge済み、配布元M `8c89a1544e9ac693d5ba388792a843baa55c9363`。reviewedHa6584bce/CI実checkout2aa64b46/Mは同一tree。CI40660526599全8/Readiness40660526626/current Codex未対応0、M push CI40672234693成功。正式40672472894の全OS build/native/Publish成功、Releaseと全5assetの実取得は確認済み。cleanup installerのみ失敗しrun全体failureであり、独立root後処理taskで修復を続ける。

## Close boundary

このchangeは、選択した3機能の現source自動検証と公開後証拠でのみ完了する。公開tag/source、5 assets/checksum、公開取得結果とcleanup結果・独立root後処理taskへの割当を記録してからarchiveする。global cleanupの未完はroot台帳で継続し、完了と偽らない。本人の実機確認をClose条件にしない。screenshotや画面の目視だけでScenario成功を主張しない。
