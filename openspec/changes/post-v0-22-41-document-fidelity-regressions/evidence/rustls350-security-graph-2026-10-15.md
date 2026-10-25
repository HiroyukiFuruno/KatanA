# Issue 350: current supply-chain repair

## Failure and scope

The normal push session 13043 terminated with exit 1 before upload. Its
preflight supply-chain stage rejected rustls 0.23.45 for
RUSTSEC-2026-0346, 0347, 0348, and 0349. Public PR 346 remained at
2a7a7107738d9918560a260403326d15749e5d4a. The raw failure is
`tmp/image-pptx-normal-push-2026-10-15.log`.

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/350

Both root and standalone screenshot locks now select published rustls
0.23.46, checksum
`48e13bd8c0e9365c43cfa5c9e8f9ad49d3c8444926c9aac819e0e4dc503c8fdf`.
Cargo also rebound cssparser-macros 0.7.1 to syn 2.0.119 and tempfile
3.27.0 to getrandom 0.3.4. Published requirements permit those selections;
these are not claimed to prove all-platform compatibility. KDV 0.5.12,
KRR 0.4.23 and KUC 0.4.1 remain unchanged registry dependencies.

## Standalone graph and durable gate

The independent screenshot graph initially failed with exit 6 for missing
MIT declarations and versionless local dependencies. Both tool packages
now declare the repository's MIT license. Screenshot local dependencies
declare their actual package versions, with normal Cargo caret semantics,
not exact registry pins. No license ignore, advisory exception, wildcard
allow, source exception, or private-package bypass was added.

`scripts/ci/check-supply-chain.sh` owns both graph checks. The Just recipe,
release preflight and Ready-only supply-chain CI call that same script.
The Draft CI condition is unchanged. Different locks each require their
own audit; manual duplicate full preflight before normal push is not added.

The release-flow contract initially failed on the absent shared checker;
after wiring all three callers, its 10 tests passed. Independent review
required `--locked` on both commands and a direct Bash Just entry rather
than assuming every RTK version accepts Bash. Both were repaired and the
10 contract tests passed again. Intermediate shared gates 56200 and 24367
exited 0. The final direct-Bash Just entry 59641 exited 0, with both graphs'
four categories OK in `tmp/rustls350-final-shared-supply-chain-2026-10-15.log`.
No static caller test is represented as an actual all-platform CI run.

## Current real input verification

Session 48062 ran the official `just test-document-fixtures` on the new
lock graph, with explicit original Office and HTML inputs. It exited 0:

- Office six originals: typed first-frame test passed, 19.00 seconds.
- Three PPTX originals: ready, next, stale-tab rejection and parent worker
  close passed, 9.42 seconds.
- Original HTML fragment s15: passed, 10.14 seconds; scroll y 3048.824,
  content height 9539, sticky sidebar ratio 0.9352.

Raw: `tmp/rustls02346-official-document-fixtures-2026-10-15.log`.
These are in-process host/worker checks, not packaged all-OS, independent
95-point fidelity, native fullscreen or performance non-regression proof.

Observed lock SHA256: root
`56bac0832157e67d9dec0c601edcd7cfd86339c09b6dc8ac877bbc10f3db5ba3`,
screenshot `ef7151771258e762d1962647ccf4625dd668d48ec3a5246b60bcb48734955896`.
Release `kdv-office-worker` SHA256:
`dc334210b1b1e466a9a961fed4cd69eb1643120b51400ccd3d5e8e484420480b`.
An initial request for a nonexistent `katana-document-worker` filename
failed; it was not a successful artifact identity check.

Current AST session 21789 passed all 23 checks, exit 0, 22.35 seconds.
Current local image loader session 73065 passed 53 tests, exit 0.
UI all-target, test-inclusive Clippy with external fixture acceptance and
`-D warnings` session 34959 exited 0 in 3 minutes 33 seconds. Official
formatting session 56356, Bash/Zsh syntax and `git diff --check` passed.
Read-only independent final review found no remaining scoped P0/P1.
The separate Issue 348 early return avoids scanning targets when no watched
directory was lost; no measured user-visible speedup is claimed.

## Remaining

Normal signed commits completed with exit 0: Issue 348 fast path
`ace75eb13d41cf12b21e778212aa606ec324b9c1`, Issue 350 lock repair
`e2901ea79bb187263a26fd15bb7e7265919e7a6b`, and the standalone locked
supply-chain gate `dd9d6dc592e2e1f3bb833e042984f37b004e3a6b`.
Each signature status was G. Normal push, fresh review, Ready CI,
packaging, public assets and postprocessing must still be collected.
A repaired supply-chain gate is not a completed release.
