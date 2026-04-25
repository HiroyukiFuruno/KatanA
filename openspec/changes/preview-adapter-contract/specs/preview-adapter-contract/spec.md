## ADDED Requirements

### Requirement: Preview is exposed through adapter-owned API

システムは、native preview を KatanA-owned adapter API 経由で公開し、parser / renderer / vendor internals を `katana-ui` から隠さなければならない（SHALL）。

#### Scenario: Render through adapter

- **WHEN** `katana-ui` が active Markdown buffer を preview 表示する
- **THEN** system は preview adapter に buffer、theme、workspace context、action sink を渡す
- **THEN** `katana-ui` は parser token や vendor fork 固有型を直接構築しない

### Requirement: Preview behavior remains unchanged during migration

システムは、adapter migration 中も現行 preview の user-visible behavior を維持しなければならない（MUST）。

#### Scenario: Render supported Markdown

- **WHEN** document が現在対応済みの Markdown、GFM table、math、diagram、anchor、emoji を含む
- **THEN** preview は migration 前と同等に表示される
- **THEN** source editor と split view の behavior は変わらない

### Requirement: Adapter returns renderer-neutral metadata

システムは、TOC、scroll sync、block highlight、search、action hook に必要な metadata を renderer-neutral DTO として返さなければならない（SHALL）。

#### Scenario: Use metadata for TOC and scroll sync

- **WHEN** preview が heading と block content を含む document を render する
- **THEN** adapter は heading anchor、block anchor、source range、rendered identity を返す
- **THEN** TOC と scroll sync は renderer internal type に依存しない

### Requirement: Preview-specific vendor usage is contained

システムは、preview-specific vendor fork API の直接利用を adapter implementation 内へ閉じなければならない（SHALL）。

#### Scenario: Vendor fork API is still required

- **WHEN** preview renderer が fork-specific API を必要とする
- **THEN** direct call は adapter implementation 内に存在する
- **THEN** `katana-ui` の外側 call site は adapter contract のみを扱う

### Requirement: Migration remains native

システムは、preview adapter migration のために WebView、React、DOM runtime、bundled web app を導入してはならない（MUST NOT）。

#### Scenario: Build desktop preview

- **WHEN** migrated preview が desktop target で build される
- **THEN** preview は Rust / egui native path を使う
- **THEN** embedded browser runtime を要求しない
