# XLSX real-input diagnostic, 2026-09-12

The reusable `scripts/screenshot/examples/xlsx-sheet-tab-input.json` fixture
opens the real two-sheet workbook and requests Dashboard -> Notes -> Dashboard
through accessibility-label pointer clicks, with bounded active-index checks.
It does not invoke DocumentNext or JumpTo directly.

Two runs fail at the first Notes selection: active_index remains 0. The first
frame is a Grid with two items (9.026 seconds cold, 0.113 seconds on the debug
rerun). The Notes node rectangle is [[349.7,839.0],[398.5,862.0]] in the
1280x900 logical viewport; the input center is [374.1,850.5].

On the debug rerun, document_surface logs show the initial frame, viewport
Resize send/apply and its returned frame, but no JumpTo send/apply after the
pointer click. This narrows investigation to host/harness input dispatch before
the viewer command; it does not yet distinguish occlusion, pointer handling,
or another host interaction issue. No timeout or acceptance threshold was raised.

This is NOT packaged-release evidence. Both runner and Office worker were built
in `target/typography-host.tyoRkw/katana` against the diagnostic KDV 0.5.6
candidate archived from 3b4ecb13396c8f067e838cacb2a13087dd88309d, registry KRR
0.4.19 and KUC 0.3.7. The diagnostic root lockfile was regenerated offline for
the worker; formal KatanA dependency files were not changed. The candidate is
not the latest KDV owner worktree and must not be described as a public release.

The runner was built with `cargo build --locked --offline --manifest-path
scripts/screenshot/Cargo.toml -j 2 --config profile.dev.debug=0 --config
profile.dev.incremental=false`. The worker used the same profile and the
runner target directory. Runtime used `KATANA_KDV_OFFICE_WORKER` pointing to
that worker and `RUST_LOG=katana_ui::preview_pane::document_surface=debug` for
the second run. Both processes exited with status 1 on the bounded assertion.

## Root cause and repair, 2026-09-12 follow-up

DEBUG-only sheet_tab_pointer records prove the Notes response is enabled,
contains the pointer, keeps the same widget ID across press/release, and has
no active drag. The actual egui interaction snapshot instead assigns the click
to the `Rendering notes (3)` collapsing header. `show_diagnostics` was called
after the grid consumed all available space, overlapping the bottom sheet rail.

`DocumentSurface::show` now allocates diagnostics before the grid and sheet
rail. The same real-pointer fixture succeeds with unchanged Notes coordinates
[[349.7,839.0],[398.5,862.0]] and Dashboard coordinates
[[258.0,839.0],[341.7,862.0]], confirming active-index transitions 0 -> 1 -> 0.
The fixed run exits 0; the preceding runs with the old layout exit 1. Its
first frame is 0.145 seconds, a warm observation rather than a cold-start claim.

Formal published-graph all-target Clippy passes with warnings denied; all 35
diagnostic harness tests pass. The DEBUG=true-only helper retains the pointer
diagnostics for future regressions. No dependency, score, timeout, or acceptance
threshold was weakened. These changes are uncommitted.

## Machine-checked geometry follow-up, 2026-09-13

The ClickNode request supports optional expected_bounds in logical coordinates.
Before pointer input, the harness requires the entire unique accessibility-node
rectangle to fit a finite, positive region inside the viewport. Unit tests
reject displaced tabs, partial overflow, empty/reversed regions, non-finite
coordinates, and a region outside the viewport.

Both XLSX fixtures now require the bottom region [250,828,1250,872] for the
fixed 1280x900 viewport. The focused real-workbook run verifies both Notes and
Dashboard bounds and completes active-index transitions 0 -> 1 -> 0, exit 0.
All 36 diagnostic harness tests and formal all-target Clippy pass. Older
requests without expected_bounds remain accepted.

Task 4.14 now has diagnostic real-input and geometry evidence, but still needs
formal published-graph/packaged acceptance. The diagnostic pass is not a release.
