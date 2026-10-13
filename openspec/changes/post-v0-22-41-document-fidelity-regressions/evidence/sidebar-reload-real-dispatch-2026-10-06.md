# Sidebar reload real-dispatch verification

## Scope and result

The new private nested test connects existing primary-sidebar pointer press and
release input to `ActionOps::take_action` and `process_action`. A real temporary
HTML file changes from red to green; the test verifies a new preview session,
actual green RGB pixels and the changed document buffer. No commercial method,
visibility, external API, test port or acceptance threshold changes.

The existing ten-second content-update contract is retained. This is not the
two-second cold-start performance test, a native human interaction result,
packaged acceptance or a visual-fidelity score. The user's reported native
reload failure remains unverified, not declared fixed by this test.

## Official checks

- `tmp/sidebar-reload-real-dispatch-targeted-test.log`: exact nested filter,
  one test passed, 1926 filtered, 40 suites, 0.33 seconds, actual exit0.
- `tmp/sidebar-reload-real-dispatch-ast.log`: 23 passed, actual exit0.
- `tmp/sidebar-reload-real-dispatch-strict-clippy.log`: strict impacted four
  packages, actual exit0.
- `tmp/sidebar-reload-real-dispatch-format.log`: package rustfmt check and
  diff check, actual exit0.

These files explicitly label captured RTK tool output. Earlier direct-entry
interruption, sibling-module layout/AST findings and unused-import/reborrow
compile failures are harness failures, not a reproduced commercial defect.
The final conventional `tests/reload.rs` module is the checked implementation.

## Self-review

PASS for the test-only concern: existing real browser startup and sidebar
helpers are reused, the queued action is asserted after primary controls input,
session identity and actual pixels are checked, and no mock, fixed sleep,
ignored test, relaxed timeout or snapshot comparison is introduced. Normal
commit/push and current-head whole release gates remain separate work.
