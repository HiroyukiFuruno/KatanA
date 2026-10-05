# Host feedback integration (2026-10-06)

## Scope

The user authorized KatanA-owned fixes for v0.22.42 and deferred KDV/KRR deficiencies to the next release. Upstream Issues are handoff only: KDV #65/#66/#67 and KRR #106/#107. No upstream implementation or unpublished dependency override is part of this change.

## Verified integration

- `f3364c4`: asynchronous local image decoding, generation cancellation, owned texture cache bounded to 16 entries / 64 MiB; five official workspace regressions passed, including 16-bit RGBA and oversize failure caching.
- `8eb40d5`: diagnostics no longer synchronously load an unloaded Markdown document on the UI thread; five real-file focused regressions passed. The supplied hang report does not establish the termination cause or prove all directory hangs resolved.
- `497392f`: forward smooth-scroll tail frames to the HTML surface without a second wheel event; the exact focused regression passed (one test). An earlier module filter ran zero tests and is not behavioral evidence.
- `04989ed`: three real HTML reload regressions passed: changed disk contents, same-content forced refresh, and preservation of unsaved contents, each bound to a new preview session and actual rendered RGB. No production reload fix is claimed: the reported native button failure remains open. Frame generation alone was an invalid initial test identity because a new session can reuse its number.
- Official AST: 23 checks passed; impacted lint passed. Full current-source coverage, cloud checks and packaged acceptance remain separate gates.

## PPTX slideshow and format-specific menus

The real document surface provides PPTX previous/next/first/last navigation. Unsupported image/document menus are disabled by format; PDF table of contents remains supported. The existing unsupported-tab regression excludes PPTX because slideshow is now supported. Formatting-only changes in that test file do not change its other expectations.

Focused worker-backed results: menu availability 5 passed, document surface 134 passed, unsupported-tab restoration 1 passed. Main inspected the code and reran the tracked request with the freshly built runner and worker:

```text
KATANA_KDV_OFFICE_WORKER=$PWD/target/release/kdv-office-worker
target/screenshot-harness/release/katana-screenshot
--request scripts/screenshot/examples/pptx-slideshow-navigation.json
--output tmp/pptx-slideshow-main-review
```

All 15 steps completed with exit 0. Actual frame indices changed 0 -> 1 -> 0 -> 1. Main's first frame was 14.951 seconds; the earlier worker run was 2.433 seconds. This variation is not evidence of stable load-time improvement. The request uses state-based document-frame assertions; its 15-second wait is a first-frame deadline, not a fixed interaction sleep. Launch, baseline and key waits are zero.

Identity SHA-256:

- Worker: `6b0676b83f24500e5f9af766c60bcec14643ba09c128cd86f698f21b70a2c72a`
- Runner: `4bbc8c4113e84a3c1b641f31001c89d4a47549471f80f62ff4fac7cecaabc2c3`
- Request: `e489eb2693e2b0691d850808509bb6967e97d601527f7515976a705527cebd3e`

Execution is in-process (`packaged_binary_tested=false`), not packaged/native or clean-machine acceptance. Escape is followed by tab-selection and active-document assertions, not a dedicated modal-visibility assertion. Same-frame keyboard-plus-button dispatch is prevented in source but has no direct E2E proof. No private document contents are included.

## PDF view-tools regression repair

The normal push of `55ef95c2` failed before upload: 1,075 native tests passed and two existing PDF view-tools tests failed. The format-specific menu implementation had grouped `Tools` with Markdown-only `Export` and `Story`, inadvertently disabling PDF split/code-preview commands. This is a KatanA regression, not an upstream blocker.

The new exact `pdf_keeps_view_tools_without_markdown_only_menus` regression failed on the old implementation. `Tools` is now separated from `Export`/`Story`, preserving PDF tools while keeping unsupported image/HTML/Office tools disabled. The official workspace `just T=view_tools test-specific` then passed five tests across 40 suites (1,876 filtered out, 0.43 seconds), including both existing failures and the new regression. Tests were not deleted or weakened.

Self-review: PASS for the focused repair. The shared availability call sites in direct dispatch and command inventory both use `PreviewMenu::Tools`; sidebar uses the same contract. No new API, threshold, test exclusion, dependency or synchronous work is introduced. The prior failed push and focused GREEN do not constitute full-gate or publication success; normal signed integration and another normal push remain necessary.

## Remaining release obligations (current)

Normal push and new-HEAD review/CI, full coverage/supply-chain, all five packaged assets/checksums and clean-machine acceptance remain required. Known upstream HTML fidelity, loading and restart-cache limitations must be disclosed, not marked fixed. Release scope must be reflected explicitly in the task and acceptance contracts rather than bypassing checks or falsely completing deferred work.
