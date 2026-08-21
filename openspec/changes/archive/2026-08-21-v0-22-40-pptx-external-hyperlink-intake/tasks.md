## 1. Provenance and boundaries

- [x] 1.1 Published KDV `0.5.4`, its GitHub Release, and registry checksum are independently confirmed before KatanA intake.
- [x] 1.2 Design and release contract fix `KatanA -> KDV -> KUC/KRR`, prohibit a direct KUC or document-engine dependency, and prohibit path/git sources.
- [x] 1.3 The v0.22.39 -> v0.22.40 adjacent SemVer rule rejects withdrawn v0.29.0 and every non-adjacent target.

## 2. Consumer intake

- [x] 2.1 Workspace version, internal crate versions, bundle metadata, and EN/JA changelogs are synchronized to v0.22.40.
- [x] 2.2 KatanA resolves exact registry KDV `0.5.4`; Cargo.lock and screenshot lock resolve official `office2pdf 0.6.7` and no retired package, path, or git source.
- [x] 2.3 The multi-format release contract is updated to make v0.22.40 / KDV 0.5.4 / official office2pdf provenance mechanically required.

## 3. External hyperlink PPTX regression

- [x] 3.1 A test-only fixture generator adds one standards-defined OOXML external hyperlink relationship to a representative PPTX without becoming application code or a document parser.
- [x] 3.2 The KatanA headless document surface opens that local fixture and requires the initial PPTX `Page` frame rather than a KDV preflight error.
- [x] 3.3 macOS, Linux, and Windows release CI execute the external-hyperlink PPTX scenario together with the existing PDF / DOCX / XLSX / PPTX corpus.

## 4. Verification

- [x] 4.1 Format, clippy, AST lint, workspace tests, strict coverage 100% / uncovered 0, and contract self-tests pass without threshold or exclusion changes.
- [x] 4.2 OpenSpec strict validation, release preflight, package / sidecar asset contract, and the exact adjacent SemVer guard pass.
- [x] 4.3 Review the actual headless evidence for the external-hyperlink PPTX on macOS, Linux, and Windows; verify a rendered page and no diagnostic surface.

## 5. Release

- [ ] 5.1 Verified evidence is recorded; then commit, push, PR, CI, merge, v0.22.40 release, and public GitHub artifact verification are completed in order. delegation-exception: `直列のクリティカルパス`
