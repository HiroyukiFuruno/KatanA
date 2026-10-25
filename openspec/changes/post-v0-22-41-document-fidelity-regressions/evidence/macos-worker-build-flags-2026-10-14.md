# macOS worker build flag alignment

Issue343 partial implementation: the direct screenshot entry now sources a small environment helper. On macOS only, unset RUSTFLAGS receives the same `-D warnings` default as Just. Explicit empty/custom flags and CARGO_ENCODED_RUSTFLAGS are preserved. Other platforms keep the old unset behavior, so Cargo target-specific settings (including Windows crt-static) are not newly overridden. No lint/coverage/test/acceptance threshold or check is removed.

The real original acceptance on frozen17e9d146 completed first; source edits started only after root session28186 exit0. Default environment alignment is separate from original acceptance and cannot rewrite its source provenance.

- Actual Bash environment tests:4 passed (including unset/empty/custom/encoded and Linux/MSYS/Cygwin branch preservation). Branch selection tests do not claim execution on those OSes.
- Existing screenshot discovery including new tests:17 passed,7.264s, session24125 exit0; raw `tmp/issue343-build-flags-screenshot-tests-2026-10-14.log`, SHA2561ba1afc8e845a36cb1a9970f15444d3112c6d25caa56c06956da167a5a5d5c02.
- bash syntax, diff check and independent readonly final review passed; Windows-setting P2 was repaired by macOS-only scope.
- Same helper/default, locked optimized worker build session12388 exit0,3m48s. Native dependencies rebuilt; raw `tmp/issue343-strict-worker-warm-build-2026-10-14.log`, SHA25697ab315ed49debe876317a0b5e87ce8815bdb0b6553985c3bc9bc364ed5e1df2.
- Second identical helper/default worker build session79251 exit0,1.97s, no compile lines; raw `tmp/issue343-strict-worker-second-warm-build-2026-10-14.log`, SHA256068578614b4818c8bfd02c315dcb93261a0012bf0e90f4c23a4b6315eb764d8c.
- Actual resulting worker SHA remains f25a13043281165c730b30d7195fee077753f902186223ef2f0653d054fa940f.

The historical full strict build30m03s and these warm observations are not a controlled whole-pipeline before/after benchmark or a guaranteed speedup. Existing fingerprint records also differ in PKG_CONFIG_PATH; this change neither pins machine-specific paths nor claims every native environment matches. The screenshot harness is a separate target/lock and still requires its own build. No all-OS cache result, native/coverage equivalence, protected CI reuse, full release timing, packaged performance or whole Issue343 completion is claimed.
