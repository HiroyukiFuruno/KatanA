# Canvas cancellation and diagnostic contracts

Scope: [Issue258](https://github.com/HiroyukiFuruno/KatanA/issues/258).

## Observed full-gate failure

The official inventory-enabled `just JOBS=2 coverage` at clean published commit
`ce95c02a057c893be865d5caecd54c28a567da85` exited 1 (session 76455).
All four test phases exited 0, including UI lib 1308 passed / 2 existing ignored,
UI parallel 144 passed / 2 existing ignored, and UI serial 18 passed. Export
regression passed 13 tests in 252.72 seconds with the original fixture skip.
The final meaningful-line gate rejected 29 unexecuted lines, primarily Canvas
serde delegation, error formatting, a non-file label branch and cancellation
classification. This is a real coverage failure, not a failing product-behavior
assertion. Strict document coverage was not reached.

The four diagnostic manifests recorded 8 / 25 / 1 / 1 test executables, matching
native listings without listing reasons, unchanged source, and exit 0. They
retain `complete=false` and `reuse_authorized=false`; they do not authorize
removing tests. Source guard verified the same clean HEAD after execution, and
both lock hashes were unchanged. Raw log:
`tmp/ce95-full-inventory-coverage-2026-10-15.log`.

## Bounded change

- Real serde JSON string/value deserializers exercise human-readable mode,
  unit/option/newtype, invalid-type expectation, unknown/known sequence size
  hints with full element consumption, and explicit cancellation errors.
- Typed Canvas errors assert exact bounded diagnostic strings. Non-file
  content variants assert that they do not expose file labels.
- Cancellation classification matches explicit cancellation first, then serde
  errors that actually observed cancellation, then ordinary input errors. The
  old early observed-cancellation return made the later explicit cancellation
  arm unreachable. The parser's final check prevents a successful document
  after observed cancellation, so the result semantics are unchanged.

No public API, settings, worker count, input limit, allocator, coverage exclusion,
threshold, ignore or test port changed. No user application was launched.

## Targeted verification and review

- Canvas targeted instrumented tests: final session 37926 exited 0, 40 passed.
  Raw `tmp/canvas-coverage-contract-final-2026-10-15.log`.
- Impacted Clippy: session 91493 exited 0, covering core/linter/platform/UI.
- Test-inclusive core strict Clippy: session 93428 exited 0.
- Official AST: session 6630 exited 0, 23 passed; formatting and diff checks pass.
- An initial invocation misplaced the test filter and did not execute tests.
  A later strengthened test compile rejected a redundant unused import; it was
  removed, without changing lint policy. Neither is a product-behavior RED.
- Independent review confirmed cancellation equivalence and meaningful real
  serde assertions. Its tentative enum P1 was withdrawn after tracing actual
  production use: the private wrapper reads only RawDocument maps/JSON values;
  typed model conversion uses ordinary `serde_json::from_value`.

The wrapper does not delegate enum access and is not a general-purpose serde
adapter. Extending production use to typed enums requires explicit enum/variant
contracts first. No current Canvas public-input failure was identified through
that unused path, so generic enum support is not added to this release slice.

## Remaining verification

Mixed earlier profiles plus targeted tests are not a fresh full-gate result.
Normal signed integration, one normal push hook, current-HEAD full coverage,
fresh review and all release/package/publication requirements remain separate.
The failed full-run log is preserved; this document does not claim that the
29-line failure, all-OS acceptance or Issue258 is already complete.
