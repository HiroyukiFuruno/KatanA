# Localized font aliases and filter-mutation queue review

Current public review of f8cb595e identified comments4177495989 (threadPRRT_kwDORm09y86oyJyZ) and4177495994 (threadPRRT_kwDORm09y86oyJyb). Both are KatanA-owned repairs; upstream publication is not a reason to postpone them.

## Bounded mutation queue

The final old-source negative control fails all three new queue regressions while five existing tests pass: `tmp/filter-mutation-bounded-old-red-final-2026-10-04.log`, exit101. ApplyValues/Clear were silently evicted by ordinary navigation pressure. The repair protects queued mutations and preserves their FIFO order. The existing maximum16 ordinary commands plus one latest Candidates request is unchanged. When all16 slots contain protected mutations, a new command is explicitly rejected and reported as an enqueue failure; existing pending/in-flight work is not cleared. There is no unbounded queue or release-budget change.

Changed-source document-surface tests pass130/130 with the real published-KDV worker prerequisite: `tmp/filter-mutation-green-final-2026-10-04.log`, exit0. This includes actual caller/channel rejection, generation, lifecycle and spreadsheet interactions. The initial focused run failed compilation while a disjoint font test was being edited; `tmp/filter-mutation-green-2026-10-04.log` is not passing evidence.

## Font alias resolution

The deterministic fixture uses the existing embedded Ubuntu font's real SFNT/name metadata with distinct preferred typographic, English legacy and Japanese same-ID legacy names. Old production resolution rejects the Japanese alias after the fixture assertions succeed: `tmp/font-alias-final-fixture-old-red-2026-10-04.log`, exit101. Earlier fixture/import failures are not behavioral negative controls.

The repair retains all valid aliases, preserves preferred canonical naming and style/weight selection, and returns every matching request for a shared face. Payloads are shared by Arc for the same face/index instead of duplicating standalone or TTC font data. Case-insensitive duplicate, empty/control names and simultaneous canonical/localized requests have explicit assertions.

The first changed-source office-face run passes24 tests but fails the new real-font regression: `tmp/font-alias-green-final-2026-10-04.log`, exit101. Its fixture requests retained trailing padding while production metadata intentionally trims names. Normalizing the fixture's parsed expectations preserves production behavior; all25 office-face tests then pass in `tmp/font-alias-green-normalized-2026-10-04.log` and the final restored-source `tmp/font-alias-green-final2-2026-10-04.log`, both exit0.

The normalized fixture was additionally checked against a canonical-only negative control (restrict alias iteration to its first canonical entry). It fails at actual resolver output with `family="日"` unavailable, not at fixture parsing or compilation: `tmp/font-alias-normalized-canonical-only-red-2026-10-04.log`, exit101. An earlier negative-control attempt produced an unused-field compilation error and is not behavioral RED evidence. The complete alias repair is restored.

Official format and AST now pass, AST23/23 in `tmp/review-pair-ast-final2-2026-10-04.log`. Initial unnamed SFNT offsets were replaced by named constants; the enqueue-failure helper was moved beside existing worker/channel failure handling to preserve cohesive responsibilities and the200-line file limit. No exclusion or rule relaxation was introduced.

## Remaining gates

Fresh focused font success, official AST/Clippy, changed-source full coverage, signed formal history, normal push, individual review replies/resolves and fresh full review retrieval remain required. The prior f8cb595e full coverage does not validate these product changes. Original-file, packaged/clean-machine, source-renderer fidelity and public-release requirements remain unchanged and unfinished.

## Pre-commit self-review

Targeted verification PASS: document surface130, office faces25, official format, AST23 and workspace Clippy (`tmp/review-pair-lint-final-2026-10-04.log`, exit0). The first Clippy run rejected a redundant closure; the function reference repair passes without disabling the rule. Call sites of the private queue bool and private multi-match return type were traced; public APIs, locks, registry identities, generation handling, worker lifetimes, queue limits and release criteria are unchanged. Real-font payload sharing and actual channel failures have concrete assertions. Tests are not ignored, deadlines and coverage exclusions are untouched, and no source renderer score or packaged acceptance is inferred. Full release verification remains pending and official changed-source coverage is running in `tmp/review-pair-full-coverage-2026-10-04.log`.

Signed concern-separated commits63279a37 (bounded mutation queue) and019d7f59 (localized/multiple aliases) are now formal local history; both signatures verify as G. They have not yet passed normal push/current review or release gates. Master remains clean, stash0, existing master/release worktrees only. A fresh worker rebuild after these changes is running in `tmp/review-pair-release-worker-2026-10-04.log`; the successful prior worker is not substituted for this new source.
