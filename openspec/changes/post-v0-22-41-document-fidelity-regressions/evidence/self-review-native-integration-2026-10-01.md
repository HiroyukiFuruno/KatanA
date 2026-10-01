# Self-review: native document-fidelity integration

## Scope

KatanA-owned Explorer projections, preview availability, sheet/filter controls,
source ownership, bounded fonts, diagnostics, startup heartbeat and published
dependency adoption. This review does not approve publication or replace the
required packaged and canonical acceptance.

## Resolved findings

- Fresh Terms-modal frames now record completed UI progress without accepting
  Terms automatically; a real isolated child-process regression verifies it.
- Worker source bytes are transferred to the session instead of retaining an
  extra clone. A real PDF open/frame/close test verifies consumed bytes and
  retained URI/revision.
- Initial standard fonts are not installed twice. A real OS-font/egui test
  verifies custom-to-standard restoration, not just the transition flag.
- Default popup CloseOnClick discarded filter selections. Only the filter menu
  uses CloseOnClickOutside; actual input tests verify Apply/Clear, disabled Apply
  for truncated candidates, and selection persistence.
- Persisted NonBlank metadata initially became an empty ApplyValues set. The
  regression fails before the minimal candidate projection fix and passes
  afterward; KatanA does not duplicate KDV's filter engine.
- The proposed parent-grid click conflict does not reproduce: actual RawInput
  requests Candidates without SelectAt. No speculative product fix was applied.

## Verification boundaries

- Real worker Candidates/Apply/Clear on an XLSX fixture passes with visible row
  counts 4 and 7. Twelve filter tests plus the parent-grid input regression pass.
- Earlier post-filter Linux UI tests pass (871 passed, two existing ignores).
  Latest Windows cross-check including test code passes.
- Latest full Linux workspace tests, native all-target strict Clippy and AST
  checks pass. MathJax typecheck/build and format/diff checks also pass.
  The native `just check` attempt failed on disk capacity, not test assertions.
  Its inactive dev cache was cleaned with Cargo; no source/worktree was deleted.
- Prior native coverage has zero meaningful uncovered lines and document-surface
  coverage of 100%, but it predates the new filter implementation. The unchanged
  full coverage gate must be rerun; no exclusion or threshold is weakened.
- Developer-Mac release-main empty-workspace smoke measures peak RSS 238752 KiB,
  27093388 font bytes and zero Office workers, with progressing UI heartbeat.
  This is not clean-machine packaged or normal-close acceptance.
- Public KDV 0.5.7/KRR 0.4.21/KUC 0.3.17 resolve from the canonical registry with
  one V8 152.2.0. Public KRR still times out on the supplied HTML initial frame.

## Remaining release findings

Full post-change coverage and quality gates, egui 0.36.2 migration disposition,
supplied HTML public-version acceptance, packaged input/clean-machine targets,
canonical fidelity scores and final current-HEAD PR review remain open. The
task ledger remains authoritative; integration commits are not release assets.

The packaging review also identifies missing live Office-sidecar identity:
empty-workspace smoke correctly requires zero workers, but cannot prove a
worker's running path/hash. Task 4.13 remains open for real packaged Office
execution rather than treating archive architecture checks as that evidence.
