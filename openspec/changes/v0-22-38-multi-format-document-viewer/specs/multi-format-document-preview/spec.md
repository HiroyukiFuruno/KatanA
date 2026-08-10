## ADDED Requirements

### Requirement: PDFとOffice文書をKDV経由でpreviewしなければならない

システムは、PDF / DOCX / XLSX / PPTX sourceを公開済みKDV `0.5.1`へ渡し、KDVの統一document session、neutral frame、viewer state、capability、diagnosticsを使ってpreviewしなければならない（MUST）。

#### Scenario: local documentを開く

- **WHEN** ユーザーがreadableなPDF / DOCX / XLSX / PPTX fileを開く
- **THEN** KatanAはcanonical file URL、bytes、MIME hint、revisionをsource descriptorへ変換する
- **THEN** KatanAはsource descriptorをKDVへ渡す
- **THEN** KatanAはKDVが返すpage / document / sheet / slide artifactをdocument tabへ表示する
- **THEN** KatanAはformatをparseまたはlayoutしない

#### Scenario: direct document URLを開く

- **WHEN** ユーザーが直接取得可能なPDF / DOCX / XLSX / PPTX URLを開く
- **THEN** KatanAはscheme、redirect、content length、timeout、MIMEを検証する
- **THEN** KatanAは取得bytes、最終URL、MIME、revisionをKDVへ渡す
- **THEN** KatanAはURL contentをHTML browserまたはWebViewへsilent fallbackしない

#### Scenario: Web application URLを開く

- **WHEN** URLがGoogle Docs / Sheets / Slides、Office 365、authentication page、またはHTML applicationを返す
- **THEN** KatanAはdocument fileとしてKDVへ渡さない
- **THEN** KatanAはdirect document URLではないことをtyped diagnosticsで表示する
- **THEN** KatanAはWebViewまたはChromiumを起動しない

### Requirement: document controlsはKDV capabilityに従わなければならない

システムは、page / document / sheet / slide navigation、zoom、fit、copy、open controlsのavailabilityをKDV capabilityから決定しなければならない（MUST）。

#### Scenario: supported commandを実行する

- **WHEN** active documentのKDV capabilityがnavigation、zoom、fit、copy、open commandをsupportedとして返す
- **THEN** KatanAは対応controlを有効にする
- **THEN** KatanAはcontrol inputをneutral KDV commandとして送る
- **THEN** KatanAはformat固有engine APIを直接呼ばない
- **THEN** pointer座標を含むdocument inputはKDV commandとして送り、KatanAでhit-testしない

#### Scenario: unsupported commandを表示する

- **WHEN** active documentのKDV capabilityがcommandをunsupportedとして返す
- **THEN** KatanAは対応controlをdisabledまたは非表示にする
- **THEN** KatanAは別engineでcommandを補完しない

### Requirement: document failureを原因まで追跡可能にしなければならない

システムは、document preview failureをlayer、operation、format、document、cause、unsupported feature、resource limitへ分解し、UIとstructured logの両方で追跡可能にしなければならない（MUST）。

#### Scenario: KDVがtyped diagnosticsを返す

- **WHEN** KDVがpassword protected、corrupt、unsupported feature、resource limit、worker failureのdiagnosticsを返す
- **THEN** KatanAは原因を単一のworker停止messageへ潰さない
- **THEN** UIは短いsummaryと展開可能なdetailsを表示する
- **THEN** structured logはlayer、operation、format、document、causeを保持する

#### Scenario: URL取得に失敗する

- **WHEN** URL fetchがunsupported scheme、redirect policy、HTTP status、timeout、size limit、MIME mismatchで失敗する
- **THEN** KatanAはfetch layerと具体的causeを表示・記録する
- **THEN** KDVまたはKRR failureとして誤表示しない

### Requirement: document preview ownershipを機械検証しなければならない

システムは、KatanAがmulti-format documentのhost integrationだけを所有することをdependency declarationとsource usageの両方で機械検証しなければならない（MUST）。

#### Scenario: ownership guardを実行する

- **WHEN** KatanA `v0.22.38` release readinessを検証する
- **THEN** KatanA sourceにPDF / OOXML parser、Office layout engine、format固有rendererが存在しない
- **THEN** KatanA sourceにformat別document runtime、KDV engine session、KUC grid materialization、document hit-testが存在しない
- **THEN** KatanAはKUCまたはKDV/KUC混成crateへ直接依存せず、KUC型をsourceで参照しない
- **THEN** KatanA dependencyにChromium、WebView、PDFium、browser helperが存在しない
- **THEN** KRRにPDF / Office viewer APIを要求していない
- **THEN** productionとacceptance harnessはpath / git KDV dependencyを使用していない

### Requirement: four-format acceptanceをheadlessで証明しなければならない

システムは、PDF / DOCX / XLSX / PPTXのlocal file、direct URL、navigation、diagnosticsをheadless acceptanceで検証し、review可能な証跡を生成しなければならない（MUST）。

#### Scenario: supported corpusを検証する

- **WHEN** headless acceptanceがrepresentative PDF / DOCX / XLSX / PPTX fixtureを開く
- **THEN** 各formatのfirst frameは固定待機なしに完了する
- **THEN** page / document / sheet / slide navigation後のstateとframeを検証する
- **THEN** screenshotとmachine-readable resultをformat別に保存する

#### Scenario: failure corpusを検証する

- **WHEN** headless acceptanceがcorrupt、unsupported、oversized、invalid URL fixtureを開く
- **THEN** failure reasonとlayerをmachine-readable resultで検証する
- **THEN** applicationとviewer workerは次のdocumentを開ける状態を維持する

### Requirement: v0.22.38 releaseは隣接版と公開済みKDVだけを許可しなければならない

KatanA `v0.22.38` は公開済みKDV `0.5.1`をcrates.ioから解決し、SemVer guardは公開済み `v0.22.37` から `v0.22.38` への隣接更新だけを許可しなければならない（MUST）。撤回済み `v0.29.0` をrelease targetとして受理してはならない（MUST NOT）。

#### Scenario: v0.22.38 release readinessを検証する

- **WHEN** release gateがtarget `v0.22.38` とlatest published `v0.22.37` を検証する
- **THEN** release gateは隣接patchを受理する
- **THEN** KDV dependencyはcrates.ioの `0.5.1`である
- **THEN** package metadataとlockfileにpath / git KDV overrideがない

#### Scenario: invalid release targetを拒否する

- **WHEN** release gateが `v0.22.37`、`v0.22.39`、撤回済み `v0.29.0`、minor jump、major jumpのいずれかを検証する
- **THEN** release gateはtargetを拒否する
