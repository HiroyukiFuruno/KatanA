## ADDED Requirements

### Requirement: workspace shellはmulti-format document sourceをdocument tabへrouteしなければならない

システムは、workspace explorer、standalone file open、direct document URLからPDF / DOCX / XLSX / PPTXを選択したとき、同じmulti-format document preview policyを適用してdocument tabへrouteしなければならない（MUST）。

#### Scenario: explorerからsupported documentを開く

- **WHEN** ユーザーがworkspace explorerでPDF / DOCX / XLSX / PPTXを選択する
- **THEN** shellはformatに対応するdocument tabを開く
- **THEN** shellは同じcanonical sourceのduplicate tabを作らず既存tabをactivateする
- **THEN** shellはMarkdown editorへbinary documentを読み込まない

#### Scenario: standalone supported documentを開く

- **WHEN** ユーザーがworkspace外のPDF / DOCX / XLSX / PPTXをfile openで選択する
- **THEN** shellはtemporary workspace policyを維持してdocument tabを開く
- **THEN** shellはtemporary workspaceをpersisted workspace tabへ追加しない

#### Scenario: direct document URLを開く

- **WHEN** ユーザーがsupported direct document URLを開く
- **THEN** shellはcanonical final URLをtab identityとして保持する
- **THEN** shellは取得済みsourceをmulti-format document previewへ渡す
- **THEN** navigation historyにredirect前URLとfinal URLの対応を保持する
