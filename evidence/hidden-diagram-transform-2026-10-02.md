# Hidden diagram transform regression

PR #346 current review of `cadfb871` identified that disabling diagram controls
blocked new gestures but still consumed the saved zoom and pan during painting.

The real egui painter regression captures image mesh vertex positions, not a
visual snapshot. With the previous product code, saved zoom 3 and pan (100, 200)
changed the vertices from `(4999.5, 72)` through `(5000.5, 73)` to `(5099.5, 272)`
through `(5102.5, 275)`: the test failed with exit 101.

The product painter now uses neutral transforms when controls are hidden while
retaining the texture and saved transform. The new regression checks unchanged
mesh positions, texture ID reuse, and preservation of zoom and pan. The existing
hidden-gesture and visible-gesture regressions also pass: three tests, exit 0.

Logs: `tmp/hidden-transform-red.log`, `tmp/hidden-transform-green.log`.
Strict all-target Clippy, AST, format and final full coverage pass. Coverage log:
`tmp/font-scheduler-full-coverage.log` (exit 0; strict document 100%, uncovered 0).
Normal push, current review reply and release acceptance remain separate checks.

## Self-review

### No issues

- Only painting with hidden controls ignores the saved transform. Existing
  visible controls, texture lifetime and saved zoom/pan are preserved.
- The regression compares real mesh coordinates and texture identity; no
  snapshot comparison or acceptance threshold changes were introduced.
- The previous code fails the regression and the candidate passes all three
  focused gesture/paint regressions, strict Clippy, AST and format checks.

### Conclusion

Targeted verification and full coverage: PASS. Normal push and current review
completion remain pending; release acceptance is not complete.
