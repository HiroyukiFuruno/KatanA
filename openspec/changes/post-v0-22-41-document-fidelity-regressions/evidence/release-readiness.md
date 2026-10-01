# Release readiness evidence

## Current boundary (2026-10-01)

Native document/Explorer/font/filter fixes and background intake are in formal release history. Packaged architecture/cwd/identity/heartbeat/all-platform publication and artifact-collection guards are integrated in `784f9e57`; the in-process input/geometry/resource harness is integrated in `80266034`. Current public dependencies are KDV0.5.7/KRR0.4.21/KUC0.3.17 with one V8 runtime. Latest focused and real-input contracts pass, but full post-change coverage, egui0.36.2 migration disposition, supplied HTML/public KRR and all packaged acceptance remain open. No release or Ready PR is claimed. Earlier dated sections below are historical snapshots, not current dependency versions or blocker status.

## Renderer-source gate regression (2026-09-07)

- The renderer contract previously indexed packages by name, silently replacing
  an earlier path/git package when a registry package with the same name followed
  it. The added test reproduced four missed rejections before the fix.
- Validation now checks every resolved KDV/KRR package. Both renderers, path and
  git sources, and both package orders are covered by eight subcases.
- `python3 -B scripts/release/test-render-dependency-contract.py`: six tests pass
  after the fix; `git diff --check` also passes. The test remains wired into the
  existing release preflight. These are contract-unit results, not proof that
  the final published dependency graph or packaged application passes.
- Live GitHub verification still shows KDV v0.5.5 and KRR v0.4.19 as latest.
  KDV PR #50 at `3b4ecb13396c8f067e838cacb2a13087dd88309d` has all three OS CI
  jobs passing but preflight failing. KRR PR #72 at
  `78ce0913d40eefc7854e409ac76822f378b918dd` has four OS CI jobs and preflight
  passing but its review latch failing. Both remain Draft; neither is a release.
  Owners were asked to continue and report publication. Final adoption must
  verify the fixes for KRR #73/#74/#76 and KUC #35, not only version numbers.
- An owner handoff later described the XLSX filter API as available in 0.5.5.
  Inspection of `document_session_types.rs` streamed directly from
  `https://static.crates.io/crates/katana-document-viewer/katana-document-viewer-0.5.5.crate`
  disproved this: `DocumentFrame` has no `spreadsheet` field. The local crates.io
  0.5.5 source also lacks the filter command export and session method. The owner
  was asked to correct the version attribution; the candidate API must not be
  treated as a usable 0.5.5 contract. Task 3.3 remains incomplete.

## Clean-machine startup matrix

The release workflow builds one universal macOS archive, then downloads and
launches that same ad-hoc-signed archive on native `macos-15` arm64 and
`macos-15-intel` x86_64 runners. Linux and Windows packaged smoke jobs remain
required by the publish job. The local startup-contract test passes seven
architecture/minimum-OS verifier tests and rejects a publish job that omits any
of these smoke dependencies.

## macOS distribution policy

Paid Apple Developer ID signing and notarization are explicitly out of scope.
The workflow retains the existing ad-hoc signature and does not reference
Apple certificate, Apple ID, team ID, or notarization secrets. The packaged
startup contract rejects reintroducing those credentials while preserving the
universal-binary and native clean-runner smoke gates. Gatekeeper or quarantine
steps required by ad-hoc distribution remain documented rather than being
misreported as a release blocker.

## Local platform verification (2026-08-30)

- `just check-light`: passed. The KatanA unit and integration layers completed
  with 867 unit tests passed (3 ignored), 140 parallel integration tests passed
  (2 ignored), and 18 serial integration tests passed.
- `just check-windows`: passed against `x86_64-pc-windows-msvc`, including all
  workspace test targets.
- `just check-full`: formatting and workspace Clippy passed, then the macOS
  fixture link reached the already-recorded duplicate-V8 ABI failure in
  `v8-dependency-link-gate.md`.
- `just check-linux`: compilation reached native test linking, but the
  7.7-GiB Docker environment terminated `ld` with signal 7 (`Bus error`). A
  serial `cargo test -j 1 --workspace` retry reproduced the same linker-process
  termination, so this run does not establish a Linux code failure or a passing
  Linux gate. It must be rerun in the release CI environment with its declared
  resources after the published KDV graph is adopted.

These results close the KDV-independent Windows and light-check work. They do
not mark the full cross-platform or packaged-startup tasks complete.
