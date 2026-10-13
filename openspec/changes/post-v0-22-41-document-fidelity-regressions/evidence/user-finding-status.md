# User finding status ledger

This ledger is intentionally stricter than the nine-item release summary: it
lists every distinct finding raised across the supplied-file review so earlier
items cannot disappear when later regressions are discussed.

Current state refreshed on 2026-10-01. Older diagnostic graphs and screenshots are not release acceptance. Native fixes are committed in `a296e49e` and `437f6c81`; the released graph is KDV0.5.7/KRR0.4.21/KUC0.3.17.

| Finding | Current status | Release evidence still required |
| --- | --- | --- |
| Explorer highlights the currently open PDF/Office file | Implemented and covered by active-reference-document selection tests | Packaged acceptance with the final published graph |
| XLSX sheets use bottom tabs instead of page navigation | Implemented; current public graph real-input/rail geometry verifies 0→1→0 | Packaged application input acceptance |
| Office table-of-contents action is inactive | Implemented across rendering, hover, shortcut, and dispatch paths | Final packaged menu interaction |
| PDF outline expands and navigates | Existing typed outline hierarchy/page-destination projection passes | A supplied PDF fixture is not present in the reported folder |
| HTML/Office download, slideshow, story, tools/view and other inapplicable menus are inactive | Implemented from one capability contract; unit and hit-target integration tests pass | Final packaged menu interaction |
| HTML action and vertical scroll input work | Historical input evidence exists, but the supplied page has no initial frame within 60s on current KRR0.4.21 | Published KRR fix and repeated original-file input acceptance |
| Supplied HTML sticky/CSS/JavaScript matches Chrome | Current original-file host acceptance is blocked before the initial frame | KRR #95/public0.4.22, then Chrome differential including active TOC |
| XLSX filter UI and behavior exist | KDV0.5.7 API adopted; KatanA projection committed, 12 tests and real worker Apply/Clear rows4→7 pass; post-egui full coverage100% | Packaged input acceptance |
| DOCX/XLSX/PPTX legal ZIP variants open safely | Current public release worker opens all six supplied XLSX/PPTX files; latest data-descriptor DOCX host reaches Page1/2 in2.026s and closes to idle | Final packaged ZIP-variant acceptance |
| DOCX/XLSX rendering approaches the source renderer | Unresolved; screenshots prove non-blank output only, not geometry parity | Agreed source-renderer references and objective missing-element/geometry scores from KDV #48 |
| PPTX first display is acceptably fast | Real main cold source read takes ~13s independently of conversion; background intake committed with cancellation/revision/FIFO UI regressions | Revised real-main/packaged first-frame, heartbeat and fidelity acceptance |
| Empty workspace and repeated previews do not cause memory/freeze regression | Font/Explorer/duplicate input-byte fixes committed. Latest actual native main empty workspace peaks247MiB with advancing UI heartbeat, fonts27MB and no Office worker; pre-egui in-process ten warm cycles add1184KiB and retain0 resources | Final packaged RSS/footprint/normal-close verification; one native PC does not prove all PCs |
| Explorer stays responsive and vertically scrollable with Office/PDF files | Verified with 10,001 rows: 14.534 ms first frame, ten interactive rows, no full-tree per-frame clone | Packaged smoke confirmation |
| Packaged app starts on every declared PC | Architecture and smoke gates implemented; current v0.22.41 macOS asset proved arm64-only; paid Apple signing/notarization is explicitly out of scope | Clean-machine matrix and public release artifacts using the existing ad-hoc macOS distribution route |
| Dependency/release graph is single and reproducible | Exact public KDV0.5.7/KRR0.4.21/KUC0.3.17 resolve one V8 152.2.0; egui0.36.2 migration committed886907a1, compatible Rust/JS updates0, full coverage/strict Clippy/supply-chain pass | Post-migration input/capture and platform/combined gates, required newer public KRR |

The change and release remain open while any row above lacks its required
evidence. In particular, a successful focused KatanA test is not a substitute
for KRR/KDV publication, clean-machine startup, or Office fidelity. The
generated DOCX closes the reproduction gap; final host and packaged evidence
remain separate from the adopted public dependency graph.
