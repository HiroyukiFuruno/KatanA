# PPTX slideshow readiness and consumer agreement

## Scope

Issue349 adopts PR346 comment4268050397 / thread PRRT_kwDORm09y86sNe-U as a KatanA host defect. KDV/KRR implementation and the published registry dependencies remain unchanged. This is not an Office fidelity or packaged-release completion claim.

The active document path must match a cached PPTX pane with a frame containing slides and no surface failure. Action dispatch, Sidebar availability, and system-modal dispatch share that capability. Missing/initially loading/failed panes cannot enter an empty Markdown slideshow. An existing usable frame remains eligible during navigation and a non-forced refresh; Markdown behavior remains unchanged.

## Actual regressions

- Old extension-only behavior: session58207, exit101, two assertion failures for direct Action/Sidebar entry. Source changes followed the RED result.
- The third consumer initially retained the format-only guard. Session70455 reached the real show_system_modals entry with a matching unavailable pane and failed the show_slideshow assertion: exit101, 0PASS/1FAIL, 0.02s. Raw: tmp/pptx349-system-modal-red-2026-10-15.log.
- The system-modal guard now shares for_app with the other consumers. The negative regression also checks exactly one fullscreen-exit command.

## Real original inputs and the durable gate

The existing just test-document-fixtures target now includes the exact real-host PPTX regression, using the same explicitly built release worker as its existing Office session gate. No separate ad-hoc command is required for subsequent corpus acceptance.

Session85140 completed the official target with the original Office directory and original requirements HTML supplied explicitly. The captured output records all three exact tests as successful:

| Gate | Result | Runtime |
| --- | --- | --- |
| Office first frames | 6 original files, 1 exact test PASS | 19.19s |
| PPTX host slideshow | fixture0/1/2, 1 exact test PASS | 9.86s |
| Requirements HTML sticky fragment | 1 exact test PASS | 11.93s |

Raw local artifact: tmp/pptx349-official-document-fixtures-2026-10-15.log. Original names/paths are intentionally not published here. The log contains three PPTX ready/next/stale_rejected/close records, not a missing-environment early return. The wrapper pipeline exited0; its complete underlying target output is retained rather than treating tee's status alone as evidence.

Each original PPTX goes through asynchronous local intake, the release Office worker, actual frame delivery, common menu eligibility, direct Action, system-modal painting, Next to index1, rejection of a different active tab, and return to the original tab. It also checks immediate availability with an existing frame during non-forced refresh. Dropping the app restores the DocumentWorkerLifecycle count to its measured baseline within the original5s bound. This counter is not an independent OS Office-child-process inventory.

- Explicit optimized arm64 worker SHA256: f25a13043281165c730b30d7195fee077753f902186223ef2f0653d054fa940f.
- Root lock SHA256: 31e2f449f83ee4d4efbd5ad1a52209d817db9fd643ae7257cd8805127a5ee7bd.
- Screenshot lock SHA256: 8fd21cb4b180c862a02203337ac3301de921ef3214ee8b20f2be0b208f3189f4.
- Live upstream Releases remain KDV0.5.12 and KRR0.4.23, both published non-Draft/non-prerelease.

## Verification integrity and self-review

The positive test uses a declared960x640 headless viewport and an8192 texture ceiling, matching the existing image unit-test setup. The first headless attempt hit egui's default2048 ceiling for a2880x1620 texture. That fixture-configuration panic is not relabeled as a reproduced production startup failure. No production rendering limit or acceptance threshold changed.

The private-module import compile error in session7936 was corrected to the existing re-export. It is not a product RED. No mock, new testport, ignore/allow, snapshot comparison, fixed sleep, timeout extension, dependency override, or approval-setting copy was introduced.

- Session50444: the full slideshow filter with explicit real fixtures, 16PASS, exit0, 9.73s.
- Session84422: official AST23PASS, exit0, 23.48s.
- Session40513: UI all-targets/test-inclusive/external-fixture strict Clippy, exit0.
- fmt-check, diff-check, and Bash syntax verification succeeded.
- Independent final read-only review found no remaining P0/P1. New methods/functions and files satisfy the existing size rules. The pre-existing long system-modal body was not mixed with unrelated refactoring.

## Remaining

The source/official gate was integrated by normal signed commit e6292c98fa1fbc3e9486f0bd8acfcf91efc19181 (session17569, exit0). Normal push, individual thread reply/resolve, fresh current-head review, Ready required checks, full coverage/3OS/supply-chain, packaged and clean-machine acceptance, independent quality scores, Release/assets/checksums, Issue closure, archive, and cleanup remain separate obligations. This test does not prove native OS fullscreen behavior, pixel/visual fidelity, eventual forced-reload outcomes, all Office-child processes, or packaged memory budgets. Human app/Terms acceptance has not been performed by the agent.
