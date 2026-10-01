# Windows headless fixture settings JSON

## Observed failure and cause

PR346 HEAD `e69bdfa6`, CI run36924444367 / Windows job110578178671:
the original cold real-font lookup and normal tests succeeded, but multi-format
headless acceptance failed with `invalid escape at line 22 column 28`.
The request was already loaded; failure preceded document opening and output
artifact creation. Raw log: `tmp/windows-e69bdfa6-job-110578178671-2026-10-02.log`.

`fixture::build_settings_json` interpolated `dir.display()` into quoted raw JSON.
Windows drive, extended-length and UNC backslashes were not JSON escaped.
This is a KatanA harness defect, not a request-deserialization or upstream
renderer failure. Windows coverage is conditional on macOS in this workflow;
the Windows skipped coverage step is not a separate failing coverage gate.

## Repair and regression

Only the workspace-path interpolation uses `serde_json::json!` string encoding;
the surrounding literal quotes are removed. Path spelling, lossy display
semantics, workspace restoration, extension filtering and request schemas are
unchanged. No slash normalization, timeout extension or acceptance waiver.

The new round-trip regression covers ordinary Windows drives, extended-length
paths, UNC, Unicode/spaces/quotes, newline/tab, and both extension-filter branches.
The old implementation failed the actual test with exit101 and `invalid escape`.
After repair all42 runner tests passed, including real settings-file restoration;
format and diff checks passed. Independent read-only review found no repair issue.
Logs: `tmp/windows-settings-json-red-2026-10-02.log` and
`tmp/windows-settings-json-green-2026-10-02.log`.

All-target strict release runner Clippy finished with exit0. The unchanged
multi-format request ran through the existing `scripts/screenshot/run.sh` entry
point on macOS: all38 steps succeeded with exit0, including Excel Notes tab
selection, frame assertions and changed-pixel assertions. Logs:
`tmp/windows-settings-json-clippy-2026-10-02.log` and
`tmp/windows-settings-json-native-acceptance-2026-10-02.log`.
Normal commit/push, fresh current review and repaired Windows cloud execution
remain required.
String round-trip tests on macOS do not establish Windows filesystem acceptance.

The screenshot workspace is separate from root `cargo test --workspace`.
CI now runs its entire locked release test suite in its existing isolated target
on each OS before the unchanged headless acceptance step. The CI resource
contract (4 tests) and native Office-worker contract (4 tests) passed; the new
contract guards the command, target, event condition and execution ordering.
Existing full workspace tests, coverage and artifact requirements are unchanged.

## Separate observations

The audit also identified manual JSON interpolation of locale/theme/preset and
fixture filename containment as separate robustness/security audit candidates.
They are not changed or declared resolved by this path-only repair.
Office RSS, canonical fidelity, upstream publication and packaged acceptance
remain governed by the unchanged main task ledger.
