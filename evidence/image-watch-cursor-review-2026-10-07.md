# Image watch and cursor review regressions

## Current review scope

- Review5432002232 at public source fe6b8e96eca95552dafc2a84baafc41405e091b3 contains two inline P2 findings: image watcher registration4198242488 and cursor cache identity4198242505. The body says no findings; the inline findings are authoritative and remain open until repair, public reply, resolve and fresh retrieval.
- Manual-target policy threads remain separate user decisions. KDV/KRR fixes remain next-release work; published dependencies stay KDV0.5.12/KRR0.4.23.

## Cursor identity

- Private mechanical extraction preserves the original raw-pointer comparison before repair. Root execution34319 actual101: two passes, three assertion failures for hotspot changes, dimensions changes and source bitmap lifetime. Raw `tmp/cursor-cache-review-red.log`.
- Repair retains the existing `CustomCursorImage` (shared Arc, dimensions, hotspot) and compares `Arc::ptr_eq` plus dimensions/hotspot. Cache hits do not clone the source; the source is retained only when a new OS cursor is stored. Existing fallback and pointer-leave cache clearing remain unchanged.
- Root execution28210 actual0: five regression tests passed. Raw `tmp/cursor-cache-review-green.log`.
- The patched dependency is outside workspace members. The normal local test route and three-OS CI explicitly run its five private tests with katana-ui's feature union; ordinary workspace tests alone do not exercise them.
- Official-route47994 actual101 observed a partially edited image helper as unused while the image repair was being written. This is a build failure, not a cursor behavior RED or passed gate. Freeze source before the replacement run.

## Image watch independence

- Real PNG plus the existing watcher event seam reproduces unavailable watching without a fake decoder. Watch retry deadline is held beyond the existing five-second Ready deadline, preventing accidental registration recovery from satisfying the assertion.
- Root execution14901 actual101: one regression failed at the existing Ready deadline, revision1/watchedfalse/error `watch registration failed`. Raw `tmp/image-watch-optional-review-red.log`; unfiltered RTK output `/Users/hiroyuki_furuno/Library/Application Support/rtk/tee/1791307344_cargo_test.log`.
- Initial repair attempts and their failed regressions are retained below. Final local repair results do not substitute for current public CI or packaged acceptance.
- First complete loader run10748 actual101:27passes/four failures. Allowing decoding before a successful initial watch registration changed the existing watch-settled ordering; GUI wakeup, missing-parent retry and access-only invalidation regressions caught this. Preserve the initial pending-registration contract, allow independent decode on explicit watch failure, and rerun the unchanged ordering assertions. Raw `tmp/image-watch-optional-review-green.log` and unfiltered RTK output `/Users/hiroyuki_furuno/Library/Application Support/rtk/tee/1791307574_cargo_test.log`. The filename is not a passed-result claim.
- Second candidate65560 actual101:25passes/six failures. Checking initial registration before an existing cache hid valid cached results; overflow tests also incorrectly demanded an immediate Ready after intentional cache invalidation. Root restores retry→cache→initial registration→decode ordering and requires eventual real-image Ready after overflow while preserving the failure diagnostics assertion.
- Final root91798 actual0:all31loader tests pass, including cold readable PNG under unrecovered watch failure, repeated failures retaining the Ready Arc, registration recovery, stale-revision rejection, real atomic replacement, GUI wakeup, missing-parent retry, overflow and cache/display lifetimes. Raw `tmp/image-watch-optional-root-green.log`. Initial pending registration is unchanged; explicit registration failure records diagnostics/backoff but no longer prevents independent decoding. Queue/decode errors are not replaced with watcher errors.
- Official targeted1256 actual0:cursor5tests with UI feature union, AST23tests, impacted strict Clippy and workspace formatting pass. Python route/resource contracts also pass. Raw `tmp/image-watch-cursor-targeted-gates.log` and `tmp/cursor-cache-route-contract.log`. Vendor formatter reported only import ordering; this was corrected without changing behavior.

## Evidence boundaries

- Ordinary signed commits54462de7ba2fec1454ccdbb8ef88e6eac8a51e94 (image) and53937b77ca0ee37ded725c6e14aa8ae953bcab44 (cursor plus permanent local/three-OS regression route) completed with actual exit0; both signatures are G. Public push, each inline reply/resolve, fresh review retrieval and new-source full gates remain unfinished.

- Public fe6b8e96 official coverage40956 actual0: UI1144/native18/export13/parallel143/serial18 pass, existing ignored tests unchanged, meaningful coverage passes and strict document surface is100% with zero uncovered lines. Independent report-only JSON/checker actual0; SHA2566b86492d9a3e4e4ae251f85f64a5eedcca24eceaed55223158246e38b2d71cc0.
- The same public source's macOS candidate7288 actual0 is universal and ad-hoc signed, with minos13.0 and versionv0.22.42. Main SHA256371f1e758abf2a113e04363e047c2f3231a11db6f9e1488b80c91ea4b941de1d; worker SHA2566a47feed0e2cb63342cd3f53dfd711678703b9e7eb3b70d3d582fe1817e743ac.
- Neither the old-source coverage nor its candidate proves these newer repairs, native human acceptance, clean-machine acceptance or publication. No human app launch/termination or consent action was performed.
