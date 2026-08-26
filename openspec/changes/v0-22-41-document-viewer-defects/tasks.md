## 1. Host interaction and explorer state

- [x] 1.1 Keep HTML pointer, focus, and wheel input on the live browser surface and add a regression test.
- [x] 1.2 Select and reveal the active PDF/Office file in the workspace explorer for every open path.

## 2. Document controls and table of contents

- [x] 2.1 Replace XLSX page arrows with horizontally scrollable bottom tabs using KDV worksheet labels.
- [x] 2.2 Disable and reject the TOC action for DOCX, XLSX, and PPTX.
- [x] 2.3 Project KDV PDF outline hierarchy into the TOC and navigate parsed page destinations.

## 3. Published dependency chain

- [x] 3.1 Verify `office2pdf-katana 0.6.10`, KDV 0.5.5, and KRR 0.4.17 are public before KatanA intake.
- [x] 3.2 Update KatanA to exact registry KDV 0.5.5 and KRR 0.4.17 with no path/git overrides.
- [x] 3.3 Update compatible direct/transitive dependencies without lowering lint, coverage, score, or acceptance gates.

## 4. Verification and release readiness

- [x] 4.1 Run focused regressions for HTML input, explorer selection, sheet tabs, Office TOC, and PDF outline navigation.
- [x] 4.2 Exercise the supplied HTML and all six Office fixtures through the real host/downstream paths.
- [x] 4.3 Pass the repository full build, test, lint, coverage, release-contract, and packaging gates for v0.22.41.
