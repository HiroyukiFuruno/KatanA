# Windows real-font lookup diagnostic

## Observed failure

PR #346 head f7931084, run 36876961856, Windows job 110418734635 failed
only `real_background_worker_returns_result_with_identity_while_ui_paints`
with `worker result: Timeout` at its unchanged ten-second deadline.
UI 948 tests passed and two existing tests remained ignored. All nine MathJax
worker tests passed in the same Windows run, unlike the prior stack failure.

## Scoped improvement

The directory scanner obtained ordinary file attributes via both `is_dir`
and `is_file`. It now reuses `DirEntry::file_type`, preserving traversal order,
final sorting/deduplication and regular file symlink acceptance. Directory
symlinks remain unfollowed; broken symlinks are rejected. Face selection and
equal-distance tie order were not changed to prioritize filenames.

The existing DEBUG-only helper now distinguishes candidate acquisition from
face resolution. Candidate acquisition includes cache borrowing and possible
OnceLock waiting, so it is not labelled as pure filesystem scan time.
The Windows CI runs the existing real-font regression in a fresh test process
with DEBUG enabled, then keeps the full workspace suite without global DEBUG.
Its resource contract checks preserve that ordering and debug isolation.

## Verification

- Real filesystem scanner/public process-entry tests: seven passed.
- Font worker tests: five passed with unchanged real-font assertions/deadline.
- Strict locked test-inclusive platform/UI Clippy, AST 23, fmt/diff passed.
- Native actual Arial Bold: 442 candidates; acquisition 3091us, resolution
  20849us, total 24029us. This does not reproduce or explain the Windows timeout.
- CI resource contract three tests and native Office worker contract four tests
  passed with the extra diagnostic step; no existing suite was removed.

## Remaining cause evidence

The redundant metadata IO is source-confirmed. Its contribution to the Windows
timeout and any improvement remain unproven until live Windows CI. The repair
does not relax deadlines, skip tests, mock font data or claim release success.

## Self-review conclusion

The scoped source and CI contract review passes. Normal `just coverage`
completed with exit0 after these changes: platform 115, UI 978 (two existing
ignored), core 215, export 13, parallel UI 143 (two existing ignored), serial
18 passed. Meaningful uncovered lines remained zero and strict document surface
coverage remained 100%. Formal push, fresh Windows evidence and final release
acceptance remain incomplete; source-level IO reduction is not a proven timeout
resolution.

## Supplied Office source integrity

Readonly `unzip -t -qq` checks passed for all six supplied XLSX/PPTX files.
The largest supplied XLSX contains 515026878 uncompressed bytes across 20 ZIP
entries. Archive CRC/container integrity therefore passed, but this does not
prove renderer ZIP compatibility, parsing memory, latency or display fidelity.
No document contents were copied into version control.
