## 1. Planning and dependency gates

- [x] 1.1 `release/v0.22.38` branchを公開済み `v0.22.37` のmasterから作成する
- [x] 1.2 KRR正本仕様がPDF / Word / Excel / PPTX viewerをKDVへ移譲していることを確認する
- [x] 1.3 KDV `v0.4.1` multi-format viewer OpenSpecへengine feasibility、ownership、security、release orderを固定する
- [x] 1.4 KatanAの旧 `v0.22.12` WebView / PDFium案を再利用しないことをdesignへ固定する
- [x] 1.5 `v0.22.38` を使うためv0.23.0〜v0.28.0のOpenSpecを繰り上げず、撤回済み `v0.29.0` をversion-undecidedへ戻す
- [x] 1.6 KDV feasibility corpus、quality profile、engine、dependency modelについてユーザーの明示承認を確認する（2026-08-02: 実装・検証・release継続を明示承認済み）
- [x] 1.7 XLSX profileが2次元virtualized gridを必要とする場合はKUC OpenSpecと公開releaseの完了を確認する（KUC v0.3.0 GitHub Release / crates.io / tag commit `1256fdd08ecc01bcc09066180e1a05d0503ba382`を確認済み）
- [x] 1.8 KDV `v0.4.1`のstrict gate、GitHub Release、crates.io publicationを確認する（GitHub Release `v0.4.1` / crates.io `0.4.1` / tag commit `3520e7e573fe702a8fcabfd46c1e5ed71b7a8348`）
- [x] 1.9 KUC / KDV / KatanAの境界を再監査し、KUC interaction、KDV統一session、KatanA thin host設計についてユーザーの明示合意を得る。証跡: 2026-08-09 user approval / file: `design.md`

## 2. Source intake and routing

- [x] 2.1 PDF / DOCX / XLSX / PPTXのextension、MIME、magic-byte routing contractを追加する。証跡: test: `document_source::tests` / file: `crates/katana-core/src/document_source.rs`
- [x] 2.2 local fileをcanonical file URL、bytes、MIME hint、revisionへ正規化する
- [x] 2.3 direct `https` document URLへredirect、size、timeout、MIME policyを実装する
- [x] 2.4 HTML application、authentication page、unsupported scheme、MIME mismatchをtyped fetch diagnosticsへ落とす
- [x] 2.5 explorer、standalone file open、direct URLを同じdocument tab policyへ接続する
- [x] 2.6 canonical source identityによるduplicate tab抑止とnavigation historyを実装する
- [x] 2.7 binary documentをMarkdown editorまたはHTML browserへsilent fallbackしない契約テストを追加する（証跡: `crates/katana-ui/src/app/document_contract.rs`、`crates/katana-ui/src/app/url_source/`、UI integration tests）

## 3. KDV viewer integration

- [x] 3.1 crates.ioの公開済み `katana-document-viewer 0.5.2`へdependencyを更新する（crates.io artifactとregistry checksumを再確認済み）
- [x] 3.2 KDV統一document sessionをKatanA document lifecycleへ接続する
- [x] 3.3 KDV中立frameだけをdocument tabの現行egui backendへ投影する
- [x] 3.4 KDV capabilityに基づいてnavigation、index jump、zoom、fit、copy、open controlsを有効化する
- [x] 3.5 document switch、tab close、workspace closeでKDV session、worker、artifact cacheを確実に終了する
- [x] 3.6 stale generation resultを破棄し、bounded queueがUI threadを停止させない契約テストを追加する

## 4. Diagnostics and security

- [x] 4.1 layer、operation、format、document、cause、unsupported feature、resource limitを保持するerror modelを追加する
- [x] 4.2 UIへ短いsummaryと展開可能なdetailsを表示し、structured logへ同じfieldを出力する
- [x] 4.3 URL fetch failureとKDV engine/worker failureを異なるlayerとして検証する
- [x] 4.4 password protected、corrupt、unsupported、oversized fixtureの回復可能性を検証する
- [x] 4.5 macro / script非実行とexternal resource blockingをKDV capability / diagnosticsから表示する
- [x] 4.6 failure後に次のdocumentを開けることをintegration testで固定する（証跡: 37-step headless scenarioのfailure corpusと最終PDF URL recovery）

## 5. Headless acceptance evidence

- [x] 5.1 representative PDF / DOCX / XLSX / PPTX local fixtureとtrusted referenceを固定する
- [x] 5.2 direct document URLとredirectをlocal deterministic serverで検証するfixtureを追加する
- [x] 5.3 各formatのfirst frameを固定待機なしで待つheadless scenarioを追加する
- [x] 5.4 page / document / sheet / slide navigation後のstateとframeを検証する
- [x] 5.5 supported corpusのscreenshotとmachine-readable resultをformat別に生成する
- [x] 5.6 corrupt、unsupported、oversized、invalid URL corpusのlayer/causeをmachine-readable resultで検証する（証跡: `scripts/screenshot/examples/v0-22-38-multi-format-documents.json`、`openspec/changes/v0-22-38-multi-format-document-viewer/evidence/release-evaluation.json`）
- [ ] 5.7 macOS、Linux、Windowsでheadless acceptanceとartifact packagingを検証する

## 6. Ownership and quality gates

- [x] 6.1 KatanA sourceにPDF / OOXML parser、Office layout engine、format renderer、format別document runtime、KUC直接参照、document hit-testがないことをAST/source guardで検証する
- [x] 6.2 Cargo metadataとsource usageにChromium、WebView、PDFium、browser helperがないことを検証する
- [x] 6.3 KRRへPDF / Office sourceまたはviewer commandを渡していないことを契約テストする
- [x] 6.4 productionとacceptance harnessにpath / git KDV dependencyがないことを検証する
- [x] 6.5 strict coverage、AST lint、format、clippy、workspace testsを閾値緩和・除外追加なしで通す（functions / lines 100%、uncovered 0）
- [x] 6.6 dependency license、advisory、artifact supply chain、`just update`後のfull gateを通す（証跡: `deny.toml`、`just supply-chain`、`just update`、`just check`）

## 7. v0.22.38 release readiness

- [x] 7.1 workspace version、bundle metadata、CHANGELOG EN/JAを `0.22.38`へ同期する
- [x] 7.2 SemVer guardがpublished `v0.22.37` -> `v0.22.38`だけを許可するようtest fixtureを更新する
- [x] 7.3 guardが `v0.22.37`、`v0.22.39`、撤回済み `v0.29.0`、minor/major jumpを拒否することを検証する
- [x] 7.4 published KDV `0.5.2`のexact resolved versionとregistry sourceをrelease gateへ固定する
- [x] 7.5 package、release preflight、headless acceptance、platform buildを通す（証跡: macOS app bundle署名検証、KDV 0.5.2で37/37 headless steps、`rtk just check`のmacOS tests・Linux workspace tests・Windows cross-check）
- [x] 7.6 PDF / DOCX / XLSX / PPTXのscreenshot、操作結果、failure diagnosticsをユーザーへ提示する（証跡: `target/multi-format-headless-kdv-0.5.2/` の13画像とcontact sheet、37/37 headless steps、2026-08-12 visual review）
- [ ] 7.7 headless証跡とrelease gateが成功した場合、2026-08-02の明示承認に基づいてcommit、push、PR、releaseを継続する

## 8. Final verification

- [x] 8.1 `rtk ./scripts/openspec validate v0-22-38-multi-format-document-viewer --strict --no-interactive`を実行する
- [x] 8.2 `rtk just check`と `rtk ./scripts/release/check-pr-ready.sh 0.22.38 --pr-bootstrap`を実行する
- [x] 8.3 KDV/KatanA call chain、registry dependency、ownership、four-format evidenceをself-reviewする（証跡: `evidence/self-review.md`）
- [ ] 8.4 2026-08-02の明示承認を記録し、未検証項目を残さずcommit / push / PR / release workflowへ進む
