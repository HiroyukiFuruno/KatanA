# Explorer performance evidence

## Regression corpus and ownership

- The kittest corpus contains one expanded directory with 10,000 visible PDF
  files, exercising the Office/PDF-visible Explorer policy in the KatanA host.
- Explorer projection, virtualization, hit testing, and scroll dispatch are
  KatanA-owned; no KDV/KRR release is required for this path.

## Measured local result (2026-08-29)

With `DEBUG=true`, the 10,001-row projection emitted:

```text
total_rows=10001 visible_rows=10 projection_rebuilt=true
projection_bytes=684296 projection_us=1845 frame_us=14534
```

- The measured rebuild frame is `14.534 ms`, inside a 60 Hz `16.67 ms` frame.
- Only 10 interactive rows were materialized; the regression bound is 16.
- The projection owns an estimated `684,296` heap bytes for its flattened row
  index. Reusing the same workspace revision and view state preserves the same
  allocation and does not rebuild, so the steady per-frame projection allocation
  delta is zero.
- The render path resolves index paths against the borrowed workspace tree; it
  contains no recursive tree clone.
- Eight scroll operations move `file-0.pdf` out of the viewport. Clicking the
  first newly visible row dispatches `SelectDocument` for that exact PDF path,
  proving both vertical progress and hit-target alignment.
- Focused `explorer` test filter: 31 tests passed across unit/integration targets.
