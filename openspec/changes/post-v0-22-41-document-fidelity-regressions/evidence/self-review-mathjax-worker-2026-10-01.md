# Self-review: MathJax execution stack ownership

## Diagnosis

Windows run36863047888/job110371948428 aborted with STATUS_STACK_OVERFLOW
in the supported-package test. The old QuickJS8MiB guard did not allocate
the caller OS stack. Production UI also calls the synchronous conversion
API, so enlarging only test threads would hide a product-path risk.

## Reviewed implementation

- The API lazily starts one named process-lifetime worker with12MiB OS stack;
  QuickJS retains its8MiB recursion guard. Runtime/Context stay on that worker.
- Public signatures and Error variants remain unchanged. Requests own text,
  display mode and individual response channels; the job queue capacity is1.
- The initialization mutex is released before send/receive waits. Spawn
  failures are not cached as a permanent initialization failure; JS
  initialization failures leave the context empty for a later retry.
- Worker disconnection returns an explicit error, without automatic restart
  or hidden fallback. A discarded response does not terminate the worker.
- No test-only production getter, timeout extension, skip, mock or reduced
  acceptance threshold was added. Caller-side synchronous waiting remains;
  this is stack ownership repair, not a claim of asynchronous UI rendering.
- The single Context is retained for process lifetime; bounded queue capacity
  does not bound all concurrent caller-owned strings or prove memory acceptance.

## Verification

`cargo test --locked -p katana-ui --lib mathjax_backend -- --nocapture`:
9 passed. Real2MiB callers cover all five existing TeX packages and inline/
block mode. Barrier-synchronized callers verify each actual SVG response,
macro definitions do not leak between jobs, and a real unclosed environment
returns JavaScriptException followed by successful normal rendering.

AST23 tests, strict locked test-inclusive UI Clippy and formatting pass.
The normal full changed-source coverage exits0 after commit0e9a94a6:
UI968 passed/two existing ignored, core215, platform113, actual export13,
parallel143/two existing ignored and serial18; meaningful uncovered lines0
and strict document surface100%. Actual Windows
rerun and all-platform release acceptance remain pending;12MiB sufficiency
on Windows is not yet proven.

## Conclusion

Targeted source review passes. Current-HEAD cloud, combined release gates,
packaged inputs and public release are not complete.
