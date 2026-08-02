## 1. Planning and dependency gates

- [x] 1.1 `release/v0.22.38` branchを公開済み `v0.22.37` のmasterから作成する
- [x] 1.2 KRR正本仕様がPDF / Word / Excel / PPTX viewerをKDVへ移譲していることを確認する
- [x] 1.3 KDV `v0.4.0` multi-format viewer OpenSpecへengine feasibility、ownership、security、release orderを固定する
- [x] 1.4 KatanAの旧 `v0.22.12` WebView / PDFium案を再利用しないことをdesignへ固定する
- [x] 1.5 `v0.22.38` を使うためv0.23.0〜v0.28.0のOpenSpecを繰り上げず、撤回済み `v0.29.0` をversion-undecidedへ戻す
- [x] 1.6 KDV feasibility corpus、quality profile、engine、dependency modelについてユーザーの明示承認を確認する（2026-08-02: 実装・検証・release継続を明示承認済み）
- [x] 1.7 XLSX profileが2次元virtualized gridを必要とする場合はKUC OpenSpecと公開releaseの完了を確認する（KUC v0.2.0 GitHub Release / crates.ioを確認済み）
- [ ] 1.8 KDV `v0.4.0`のstrict gate、GitHub Release、crates.io publicationを確認する

## 2. Source intake and routing

- [ ] 2.1 PDF / DOCX / XLSX / PPTXのextension、MIME、magic-byte routing contractを追加する
- [ ] 2.2 local fileをcanonical file URL、bytes、MIME hint、revisionへ正規化する
- [ ] 2.3 direct `https` document URLへredirect、size、timeout、MIME policyを実装する
- [ ] 2.4 HTML application、authentication page、unsupported scheme、MIME mismatchをtyped fetch diagnosticsへ落とす
- [ ] 2.5 explorer、standalone file open、direct URLを同じdocument tab policyへ接続する
- [ ] 2.6 canonical source identityによるduplicate tab抑止とnavigation historyを実装する
- [ ] 2.7 binary documentをMarkdown editorまたはHTML browserへsilent fallbackしない契約テストを追加する

## 3. KDV viewer integration

- [ ] 3.1 crates.ioの公開済み `katana-document-viewer 0.4.x`へdependencyを更新する
- [ ] 3.2 KDV PDF / Office source descriptorをKatanA document lifecycleへ接続する
- [ ] 3.3 KDV page / document / sheet / slide artifactをdocument tabへ表示する
- [ ] 3.4 KDV capabilityに基づいてnavigation、index jump、zoom、fit、copy、open controlsを有効化する
- [ ] 3.5 document switch、tab close、workspace closeでKDV worker、cancel、artifact cacheを確実に終了する
- [ ] 3.6 stale generation resultを破棄し、bounded queueがUI threadを停止させない契約テストを追加する

## 4. Diagnostics and security

- [ ] 4.1 layer、operation、format、document、cause、unsupported feature、resource limitを保持するerror modelを追加する
- [ ] 4.2 UIへ短いsummaryと展開可能なdetailsを表示し、structured logへ同じfieldを出力する
- [ ] 4.3 URL fetch failureとKDV engine/worker failureを異なるlayerとして検証する
- [ ] 4.4 password protected、corrupt、unsupported、oversized fixtureの回復可能性を検証する
- [ ] 4.5 macro / script非実行とexternal resource blockingをKDV capability / diagnosticsから表示する
- [ ] 4.6 failure後に次のdocumentを開けることをintegration testで固定する

## 5. Headless acceptance evidence

- [ ] 5.1 representative PDF / DOCX / XLSX / PPTX local fixtureとtrusted referenceを固定する
- [ ] 5.2 direct document URLとredirectをlocal deterministic serverで検証するfixtureを追加する
- [ ] 5.3 各formatのfirst frameを固定待機なしで待つheadless scenarioを追加する
- [ ] 5.4 page / document / sheet / slide navigation後のstateとframeを検証する
- [ ] 5.5 supported corpusのscreenshotとmachine-readable resultをformat別に生成する
- [ ] 5.6 corrupt、unsupported、oversized、invalid URL corpusのlayer/causeをmachine-readable resultで検証する
- [ ] 5.7 macOS、Linux、Windowsでheadless acceptanceとartifact packagingを検証する

## 6. Ownership and quality gates

- [ ] 6.1 KatanA sourceにPDF / OOXML parser、Office layout engine、format rendererがないことをAST/source guardで検証する
- [ ] 6.2 Cargo metadataとsource usageにChromium、WebView、PDFium、browser helperがないことを検証する
- [ ] 6.3 KRRへPDF / Office sourceまたはviewer commandを渡していないことを契約テストする
- [ ] 6.4 productionとacceptance harnessにpath / git KDV dependencyがないことを検証する
- [ ] 6.5 strict coverage、AST lint、format、clippy、workspace testsを閾値緩和・除外追加なしで通す
- [ ] 6.6 dependency license、advisory、artifact supply chain、`just update`後のfull gateを通す

## 7. v0.22.38 release readiness

- [ ] 7.1 workspace version、bundle metadata、CHANGELOG EN/JAを `0.22.38`へ同期する
- [ ] 7.2 SemVer guardがpublished `v0.22.37` -> `v0.22.38`だけを許可するようtest fixtureを更新する
- [ ] 7.3 guardが `v0.22.37`、`v0.22.39`、撤回済み `v0.29.0`、minor/major jumpを拒否することを検証する
- [ ] 7.4 published KDV `0.4.x`のminimum resolved versionとregistry sourceをrelease gateへ固定する
- [ ] 7.5 package、release preflight、headless acceptance、platform buildを通す
- [ ] 7.6 PDF / DOCX / XLSX / PPTXのscreenshot、操作結果、failure diagnosticsをユーザーへ提示する
- [ ] 7.7 headless証跡とrelease gateが成功した場合、2026-08-02の明示承認に基づいてcommit、push、PR、releaseを継続する

## 8. Final verification

- [ ] 8.1 `rtk ./scripts/openspec validate v0-22-38-multi-format-document-viewer --strict --no-interactive`を実行する
- [ ] 8.2 `rtk just check`と `rtk just VERSION=0.22.38 release-check`を実行する
- [ ] 8.3 KDV/KatanA call chain、registry dependency、ownership、four-format evidenceをself-reviewする
- [ ] 8.4 2026-08-02の明示承認を記録し、未検証項目を残さずcommit / push / PR / release workflowへ進む
