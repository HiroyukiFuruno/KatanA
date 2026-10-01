# Self-review: loaded font names, Workspace compatibility and CI prerequisites

## Verified boundaries

- Loaded-font aliases borrow the existing payload and collection index. No additional
  system-font scan, payload read, per-frame registration or `set_fonts` loop is added.
- Existing named families are preserved. Regular aliases retain the existing generic
  fallback chain, font payload byte total and Arc identity.
- Ubuntu/Hack metadata and actual egui layout pass; invalid/truncated payloads and
  invalid collection indexes do not create aliases.
- The production constructor fails its metadata-name assertion with the old caller
  and passes after registration is restored. All24 font-loader tests pass, including
  real-font OS/2 metadata input cases which must not register Bold/Italic as Regular.
  These input cases do not prove actual Bold glyph rendering. The existing DEBUG-only
  helper records registration count and elapsed microseconds.
  A focused constructor run with `DEBUG=true` emits
  `event=font_families_registered added=4 elapsed_us=659`; normal focused runs emit
  no such debug record. This measures the default-font fixture, not full-app startup.
- Core `Workspace` retains the original public `root` and `tree` literal shape.
  The external integration test compiles and passes.
- Workspace revision now belongs to UI state. Production open, refresh, close and
  removal use its setter. Five projection tests pass, including real search-cache
  invalidation after a same-root tree update and unchanged-frame reuse without a
  full-tree clone.
- Windows CI job110270022852 rejected the valid Cargo target path
  `D:\a\KatanA\KatanA\target` before workspace tests. Native path validation and
  slash normalization now pass four drive/UNC/Unix/relative-path contracts.
- The Linux process regression fails with the old zombie predicate (exit1,
  lifecycle condition not reached) and passes with the repaired predicate.
- Actual Linux verification initially exposed absent `ps`. The CI image now declares
  `procps`; its rebuilt image passes all seven real-process and 23 deadline contracts.
  Both suites are included in the normal Linux gate rather than a one-off invocation.
- Native process/deadline suites pass seven/23 tests. Strict test-inclusive Clippy
  for UI/core, all-workspace formatting and the 23 AST contracts pass.

## Findings repaired during review

- Candidate font tests incorrectly treated the egui frame output as a galley,
  inserted nonexistent font keys and did not actually inspect invalid payloads.
  Tests now use actual font bytes, capture actual layout and exercise failure paths.
- Egui0.36.2 API and unapplied texture-delta ownership are handled in the test frame.
- Font metadata weight/language constants replace AST-rejected numeric literals.
- An unnecessary mutable binding in the new projection test is removed.
- The old core global revision test is replaced by UI-owned revision/invalidation
  coverage, not discarded without preserving its purpose.

## Remaining release requirements

This is targeted validation, not release acceptance. Required public KRR0.4.22 and
KDV0.5.8 remain unadopted. Real Bold-face selection, unavailable Office font families,
objective document fidelity, unchanged full coverage/quality gates, packaged input,
clean-machine startup, current-HEAD review/checks and public release remain open.
No reference image, quality threshold, coverage exclusion or existing ignore is changed.

The source/lockfiles/history/evidence and the pending screenshot-harness cache are
retained. Only inactive development artifacts of `katana-ui` were regenerated using
Cargo's package-scoped clean command (2666 files/5.9GiB; free space5.4GiB afterwards).

## Conclusion

Targeted review PASS. Formal commits are4c3f75ce (Windows path),09f39b8a
(Linux process contract),2a89aa37 (Workspace compatibility) and736f8862
(loaded font names). Normal push/current-HEAD reviews and full release gates remain
required; no release completion is asserted.
