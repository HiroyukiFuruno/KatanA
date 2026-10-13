# V8 dependency/link gate evidence

## Current registry graph

KatanA currently resolves two incompatible V8 crates through published
dependencies:

- `katana-document-viewer 0.5.5` directly pins `v8 150.0.0`.
- `katana-render-runtime 0.4.19` resolves `v8 152.2.0`.
- KDV's semver KRR dependency also resolves KRR 0.4.19, so both V8 versions
  enter the same KatanA binary graph.

## Observable failure

### Fresh 2026-09-13 reproduction

`cargo tree -i` proves that KDV 0.5.5 introduces V8 150.0.0 while both direct
KRR 0.4.19 and KDV's KRR dependency introduce V8 152.2.0. A fresh
`just check` reproduced two forms of the shared-native-output collision:

1. The first parallel build produced an invalid `librusty_v8.a` whose member
   size exceeded the archive size.
2. After cleaning only the disposable V8 build artifacts, the next build
   compiled both V8 packages but linked the V8 152 Rust rlib against the wrong
   contents in the shared `target/debug/gn_out`, producing unresolved
   `simdutf`, `std::shared_ptr`, and `v8::*` native symbols.

KatanA Clippy and type checking pass because they do not perform the affected
final native link. The unchanged full test gate therefore remains correctly
blocked until the published KDV graph contains one V8 package.

The full `just check-full` gate passed format and workspace Clippy, then failed
while linking the fixture integration test with:

```text
Undefined symbols for architecture arm64:
  "_v8__Locker__IsLocked"
ld: symbol(s) not found for architecture arm64
```

Focused KatanA UI tests and the existing already-linked test artifacts pass, but
a fresh workspace integration link is not reliable with the duplicate native
V8 archives. This is a release blocker rather than a reason to weaken or skip
the full gate. It is also a credible contributor to binary size, startup
failure on some PCs, and process memory growth.

The failed link also showed KatanA requesting macOS 11.0 while V8 objects were
built for macOS 12.0. The project documentation declares macOS 13 Ventura or
later, so KatanA now makes 13.0 explicit in Cargo's build environment,
`Info.plist`, and the packaging script. The archive verifier parses every arm64
and x86_64 Mach-O slice plus `LSMinimumSystemVersion` and rejects any value
other than 13.0; all six verifier tests pass.

The KDV v0.5.6 owner session was notified to align the direct V8 version with
KRR 0.4.19 and to prove a consumer link with no duplicate V8 entry. KatanA will
rerun this unchanged full gate only after the published KDV registry artifact
is available.

KatanA now runs `scripts/release/check-render-dependency-contract.py` from the
release preflight. It rejects multiple resolved V8 packages, non-registry
KDV/KRR packages, non-exact requirements, and path overrides. With public KDV
0.5.5 the gate intentionally fails with V8 150.0.0 and 152.2.0; KDV 0.5.6 must
make this gate green before adoption is complete.
