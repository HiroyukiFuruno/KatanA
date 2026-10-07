# Self-review: Office source intake

## Scope and findings

- `full_refresh_preview` binary path no longer calls canonicalize/read/validation/hash on the UI thread. The existing prepared/remote source API remains unchanged.
- The receiver belongs to its pane. Replacing it or switching to HTML/Markdown discards stale results; no background callback mutates UI state. Dropping a pane does not join a blocked filesystem operation on the UI thread.
- Repeated requests while the same path is loading coalesce; a forced request is retained, and a follow-up read captures a change arriving during the first read. Revision equality still preserves the existing session, while changed bytes or force replace it.
- Review found two loading-state reset cases when replacing pending work. Both are repaired. No lint/coverage/acceptance thresholds were relaxed, no reference image was changed, and no new ignore was introduced.
- Initial read delay remains an unresolved filesystem-side diagnosis. Existing original-file measurements do not prove an OS-provider or quarantine root cause.

## Verification

- Linux UI library: 878 passed, 2 existing manual tests ignored. Includes five new intake regressions using actual filesystem/FIFO/egui execution.
- Linux UI parallel integration: 141 passed, 2 existing manual tests ignored.
- Linux fixture integration: 8 passed; serial integration: 18 passed.
- Native all-target strict Clippy, format, AST 23 tests, and diff whitespace checks pass.

## Conclusion

PASS for scoped implementation integration. Full post-change coverage, real main reacceptance, clean-machine packaging and final release remain incomplete. The running old main was not stopped or replaced. This checkpoint is not a release completion claim.
