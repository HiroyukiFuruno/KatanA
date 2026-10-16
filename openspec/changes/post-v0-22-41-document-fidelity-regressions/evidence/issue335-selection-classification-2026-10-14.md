# Push Issue selection and dependency classification

This candidate slice implements actual Git commit-message selection and pure
Cargo manifest/lock semantic classification. It does not activate production
push admission or complete Issue #335.

## Verification

- Root unittest discovery with warnings as errors: session 32453, exit 0,
  113 tests, 8.917 seconds. Legacy commit-reference CLI: session 42472,
  exit 0, 17 tests, 2.378 seconds.
- Real temporary Git regressions exercise complete fast-forward ranges,
  graft/shallow rejection, same-repository commit references and oversized
  commit objects. Existing historical live lookup confirmed genuine Issues
  #335 and #339; this is not a trusted production hook invocation.
- Semantic regressions preserve workspace inheritance, aliases, versions,
  features, source kinds/selectors, multiversion removals and checksum or
  dependency changes even when lock identity is unchanged.
- Reproduced delimiter collision and ignored common-key changes were repaired.
  Fresh review also found mixed None/text source sorting; unchanged and
  checksum-change regressions now cover that boundary.
- Git rev-list itself caps output at 4097 commits, rejecting selections over
  4096 before per-object processing. Issue parsing caps individual commit
  objects at 1 MiB and numeric reference tokens at 20 digits.
- Root AST inspection of eleven candidate Python files found no functions
  over 30 lines and no files over 200 lines; git diff --check exited 0.

## Remaining admission requirements

Caller-provided default refs, new-ref bases, internal package names and workspace
manifest bytes are not provenance proof. Actual production orchestration must
bind those to trusted Git/workspace facts, cover all changed manifest/lock blobs,
verify upstream registry/publication evidence and trusted gate results, preserve
repository hook ordering/stdin, and implement publication-qualified cleanup.
Tag/default/deletion exclusions from Issue selection do not grant publication
or deletion authority. No user application or settings were operated.
