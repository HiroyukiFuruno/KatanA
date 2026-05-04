# Document Viewer Integration Spec

## Requirements

### 1. Local Document Viewing

KatanA SHALL support viewing various document formats locally.

#### Scenario: PDF Viewing
- **WHEN** A user opens a `.pdf` file.
- **THEN** The app displays the PDF content with page navigation and zoom controls.

#### Scenario: CSV Viewing
- **WHEN** A user opens a `.csv` file.
- **THEN** The app displays the data in a table format with sortable headers.

#### Scenario: Office Document Viewing
- **WHEN** A user opens a `.docx`, `.xlsx`, or `.pptx` file.
- **THEN** The app displays a read-only preview of the document.

### 2. Web Document Integration

KatanA SHALL support viewing web-based documents via URL.

#### Scenario: Web Spreadsheet Viewing
- **WHEN** A user enters a URL to a Google Sheet or Office Online spreadsheet.
- **THEN** The app displays the interactive web view of the spreadsheet.
- **NOTE** Private documents may require the user to log in within the WebView.

#### Scenario: Document URL Input
- **WHEN** A user clicks a "Open Web Document" button and enters a URL.
- **THEN** The app loads the URL in a dedicated viewer pane.

#### Scenario: Offline Handling
- **WHEN** A user attempts to open a web document while offline.
- **THEN** The app displays a friendly error message explaining the lack of connectivity.

### 3. Local Preservation of Web Documents

KatanA SHALL support saving web documents to the local filesystem.

#### Scenario: Save Web Document Locally
- **WHEN** A user is viewing a web document.
- **AND** Clicks the "Save Locally" button.
- **THEN** The app attempts to download the original file.
- **AND** If a direct download is not possible, offers to export as PDF or open in a browser.
- **AND** The saved file is added to the workspace.
