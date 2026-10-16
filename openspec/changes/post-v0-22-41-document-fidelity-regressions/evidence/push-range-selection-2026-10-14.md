# Native push metadata and immutable branch range selection

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/335

Source: `81a7ed0f54df90e5907533fd210ad5376209288f`, normal signed commit.
Final independent HIGH diff review found no remaining P0/P1 in this slice.

This slice parses bounded native pre-push metadata and selects real commit
objects for branch updates. It does not activate a production hook or prove
publication, manifest semantics, registry availability, trusted verification
execution, cleanup authority, or complete Issue acceptance.

## Actual verification

- Native metadata tests retain valid Unicode ref characters after real
  `git check-ref-format` validation. The original whitespace parser rejected
  a valid U+3000 ref; the recorded assertion failure was repaired by splitting
  the native ASCII-space/LF protocol without treating Unicode as separators.
- Real SHA-256 Git fixtures cover exact fast-forward selection, unchanged
  updates, missing/abbreviated objects, non-fast-forward rejection, deletion
  selection, tag rejection, and an explicit new-ref base requirement.
- A real A -> B -> C fixture with `info/grafts` declaring C -> A demonstrated
  that `--no-replace-objects` alone selects C and omits B. The old selector
  failed one assertion in six tests, actual exit 1. The Git reader now removes
  inherited Git variables and sets its own null graft file, preserving B and C.
- A real merge DAG with one shallow parent passed the ancestor check through
  its other parent while omitting a new commit. The old selector failed the
  typed-rejection assertion, actual exit 1 in seven tests. Range selection now
  refuses shallow repositories instead of treating an incomplete list as proof.
- Single final discovery: 59 tests, 4.837 seconds, actual exit 0. `git diff --check`
  also exited 0. These are focused Python/real-Git checks, not full Ready CI,
  native application coverage, packaged acceptance, or all-platform evidence.

Raw local logs are `tmp/issue335-native-unicode-ref-red-2026-10-14.log`,
`tmp/issue335-graft-range-red-2026-10-14.log`, and
`tmp/issue335-push-range-graft-green-2026-10-14.log`,
`tmp/issue335-shallow-range-red-2026-10-14.log`, and
`tmp/issue335-push-range-final-2026-10-14.log`.

## Trust boundaries and remaining work

An explicit new-ref base proves only a real local ancestor; the future caller
must obtain a trusted baseline. An empty selection does not prove admission or
authorize deletion. Tags are outside this branch selector. Metadata shape is
not repository identity; actual object resolution supplies that separate check.

Temporary Lefthook probes preserved the existing stage before admission and
delivered both native ref tuples to the admission stage exactly once. A prior
stage failure prevented admission and left the remote unchanged. This proves
the temporary fixture behavior, not commercial gate execution or activation.
The initial probe was accidentally written to ignored primary-workspace tmp;
it was preserved and subsequent probes were performed in the authorized release
worktree. No real shared hook or tracked primary configuration was changed.

Current-HEAD review, production dispatcher admission, live Issue and downstream
publication binding, trusted verification records, and scoped cleanup remain
unfinished. Existing quality gates and release criteria are unchanged.
