## Context

KDV v0.5.4 is a published registry release that accepts standard passive OOXML external
hyperlink relationships during Office preflight. Its Office worker remains responsible
for conversion and blocks network access. KatanA v0.22.40 is only the consumer patch
that must make the corrected KDV behavior available in the native document tab.

## Decisions

### D1. Dependency direction remains unchanged

The dependency direction is `KatanA -> KDV -> KUC/KRR`. KatanA keeps using KDV's neutral
document session, frame, and command API through the existing document surface. It does
not import KUC directly and does not own Office conversion, page/slide layout, hyperlink
semantics, hit testing, or worker lifecycle policy.

### D2. Published registry provenance is mandatory

KatanA declares `katana-document-viewer = "=0.5.4"`. Cargo.lock must resolve exactly one
registry KDV v0.5.4 and the official registry `office2pdf 0.6.7`; it must not resolve
`office2pdf-katana`, a path source, or a git source. KatanA must not declare an Office or
PDF engine directly.

### D3. Host regression uses a test-only relationship fixture

The existing representative PPTX is copied into a temporary screenshot fixture and given
one standards-defined external hyperlink relationship. The generator is test-only and
performs a fixed relationship-member byte insertion inside the ZIP archive; it is not linked
into the application and does not parse or render Office documents. The existing headless KatanA
scenario opens that file and requires a PPTX page frame. The upstream KDV contract remains
the authority that the external target is never contacted.

### D4. Release target and evidence

The latest public KatanA release is v0.22.39. The SemVer guard therefore accepts only
v0.22.40 and rejects v0.22.39, v0.22.41, withdrawn v0.29.0, and all minor or major jumps.
Before release, strict coverage must remain 100% with zero uncovered lines, OpenSpec strict
validation, macOS/Linux/Windows headless acceptance, package contracts, and release
preflight must pass. Public completion requires the GitHub Release and registry KDV intake
to be independently verified.
