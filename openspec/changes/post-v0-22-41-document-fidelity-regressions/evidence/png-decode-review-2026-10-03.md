# Complete PNG evidence validation

PR #346 P1 comment 4171390024 identified that a correctly hashed 33-byte
PNG header could pass the render-output binding. This is an evidence-validator
defect, not proof that original-file product acceptance succeeded.

The validator now requires a complete terminal IEND chunk, verifies PNG chunks
with Pillow, and reopens and loads all image data. Both the reference and
packaged measured output use the same check. Pillow decompression-bomb warnings
and errors fail closed. Existing viewport, byte-digest, run identity and geometry
contracts remain unchanged.

Pillow 12.3.0 was live-checked against the official PyPI metadata and installed
in an isolated ignored `target/release-evidence-python` virtual environment.
Normal preflight creates that environment when absent and installs the exact
binary-only requirement before its existing self-test and strict evidence gate.
It does not change the app or worker dependency graph or the global Python.

## Verification

- The new regression synchronizes every render/metrics/contract/run digest for
  HTML and Office, both reference and measured output, across five corruptions:
  header only, truncated image data, missing IEND, bad CRC, and invalid DEFLATE.
- The HEAD validator loaded into a separate module rejected none of these:
  all 20 negative assertions were RED. Commercial files were not replaced.
- An initial decoder-only full test found four truncated cases still accepted.
  Requiring complete IEND repaired that gap; the focused regression passed.
- Final normal unittest invocation passed all 55 acceptance/release-gate tests,
  with output in `tmp/png-decode-full-green.log`.
- `zsh -n scripts/release/preflight.sh`, `pip check`, and `git diff --check`
  passed. No thresholds, ignored cases, source-hash exclusions or product
  acceptance claims were changed.

## Self-review

PASS for this bounded repair: chunk verification alone is not relied upon;
actual decode is required, corrupt bytes remain rejected after digest updates,
valid logical/DPI render fixtures still pass, and ordinary preflight supplies
the decoder to the same Python executable used by its subprocess gate.
Original HTML normal-close/performance, Office fidelity and packaged acceptance,
all clean-machine targets, current review/CI, and public release remain open.
