# Current comparison-contract boundary

Current PR #346 HEAD 4947f9b9 review identified two acceptance-validator gaps:
HTML interactive differential (4170924858) and Office source-renderer fidelity
(4170924862). These are release-gate repairs, not proof of product acceptance.

## HTML

The existing supplied-file request fixes the logical viewport at 1280x900 and
navigates to #s15, but does not measure the active TOC, visible section, or Chrome
geometry differential. The historical Chrome 1440x900 and host 1162.5x741.8884
measurements in html-sticky-ownership.md cannot satisfy the same-viewport current
comparison. No numeric geometry tolerance is adopted from those measurements.

## Office

The existing KDV owner directly inspected and reverified its representative
DOCX/XLSX evidence and provided:

- Fixed reference contract SHA256:
  `23748332dd5511e08dbd7637004c7379533ff763408bb3834150a1c74cadffb6`.
- KDV 0.5.9/KUC 0.4.1 candidate measurement SHA256:
  `d368c70fe9974c7e333212ea81557fc923948e0d04f7f7a26071f00312b14c55`.
- Source-renderer reference: comparison-only LibreOffice 26.8.0.3,
  pdftoppm 26.05.0 at 72 DPI; not a production runtime dependency.

Main reread the owner's complete handoff file:
`/Users/hiroyuki_furuno/works/private/katana-document-viewer/tmp/issue59-rss.oBZVZI/office-source-reference-metrics-handoff-2026-10-03.md`.
This turn did not regenerate those candidate measurements.

Representative DOCX has page/pixel and native-dimension metrics, but no
independent element missing-count. Representative XLSX has missing/style/track
metrics against OOXML, not element geometry extracted from the source-renderer
raster. These are not the six supplied original fixtures or packaged-main runs.
Old PPTX Hayro/Poppler pixel differences lack the required original-input
identity and element-geometry/missing-element tolerance contract.

Do not copy representative tolerances into other original fixtures, interpret
normalized track deltas as pixel distances, equate pixel comparison with zero
missing elements, or reuse these records as current packaged-main acceptance.

## Validator repair boundary

Receipt-only status labels, self-declared loose tolerances and unverified
contract hashes cannot prove acceptance. Comparison validation must bind actual
contract bytes, original input, independent renderer, reference identity,
viewport and named metrics. Geometry deltas must derive from measured versus
reference geometry. Identical output bytes are allowed when the producers are
independent. Actual per-original comparison contracts and receipts remain
unmet; no production numeric tolerance or nonvisual score formula is introduced
by the validator repair.

Main integration review rejected the initial draft: different output hashes do
not prove independent producers; a hash-shaped string does not verify a real
contract; receipt-supplied deltas can hide displaced rectangles; and comparing
one viewport to itself is not a differential. Those issues must be repaired and
covered by negative and boundary tests before committing the validator.

Main also reproduced a fixture-setup hole: newly created contract JSONs did not
change the enumerated source-tree hash because the draft excluded the contract
directory. Main rejected and removed that exclusion. Versioned contracts are
now required; synthetic contracts must
be installed and tracked before the test fixture's source hash is calculated.
Writing an invalid receipt must not regenerate its reference contract or silently
repair its stale source hash.

## Verified repair (2026-10-03)

Main independently ran acceptance-evidence tests (34 passed) and release-gate
tests (16 passed), with no threshold or source-hash exclusion change. Negative
tests reject missing comparison/hash, stale contract bytes, untracked or
symlink contracts, wrong navigation/viewport/state, displaced geometry, missing
Office elements and receipt-supplied deltas. Signed coordinates and identical
reference/measured hashes remain valid. Both full verification and the direct
HTML comparison validator reject each tampered contract. Diff whitespace check
passed. Main self-review confirmed reference contracts are loaded from tracked
repository files and their bytes participate in the source-tree identity.

This verifies the evidence gate only. Original-file contracts, actual packaged
comparison receipts, independent scoring and public release remain incomplete.
