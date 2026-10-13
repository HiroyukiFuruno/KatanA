# Failed image retry and bounded HTML diagnostic summary

Baseline: public PR #346 HEAD `0fae445dc43ecf05d4a77a6d8306974d79aae1e2`.
Review: `5434903489`.

## Failed decode while watching remains unavailable

Thread `PRRT_kwDORm09y86pq2FP`, comment `4200733415`.

The existing watch-registration retry did not evict a failed decode result. A readable image could therefore remain failed while watch registration continued to be unavailable.

- Genuine runtime RED: execution `53394`, exit 101; one test passed and one failed at the expected Pending assertion. Raw: `tmp/failed-decode-review-red.log`; expanded RTK output: `1791323487_cargo_test.log`.
- The private unit controls unavailable watch-registration state, but the missing file, PNG creation and asynchronous image decoding use real file IO. This is not evidence from an actually unsupported kernel filesystem or from a packaged application.
- Candidate repair removes only failed decode entries for the requested path when the existing watch retry deadline becomes due. Successful images/textures, unrelated paths, pending work, generation, revisions, retry delays and cache byte budgets are unchanged.
- Regressions retain successful image Arc identity and byte accounting, preserve unrelated failures, and evict a failed background variant of the same path. Watch errors remain visible while registration remains unavailable.
- Initial all-loader execution `5331` exited 101: 37 passed, one existing missing-parent regression failed because image readiness legitimately preceded asynchronous watch registration. Its private wait now observes the actual red pixel and cleared watch error within the same original five-second budget. The final cleared-error assertion is retained; no fixed sleep, timeout extension or ignore was introduced.

## HTML layout diagnostic summary

Thread `PRRT_kwDORm09y86pq2FU`, comment `4200733425`.

The previous DEBUG layout summary collected and joined the whole HTML input before keeping only 80 characters, repeating large allocations at multiple trace points.

- Genuine runtime RED: execution `98960`, exit 101; three passed, one failed with `summary consumed too much tail`. Raw: `tmp/html-summary-review-red.log`; expanded RTK output: `1791323618_cargo_test.log`.
- RED used a temporary old-equivalent private iterator adapter. That full-input adapter is not part of the final candidate.
- The candidate uses a private cohesive `html/summary.rs` module. Its summary retains at most 80 Unicode characters plus an ellipsis and reads at most 4096 raw characters. It never collects the whole body or a vector of whitespace-separated words.
- Short URL, Unicode, normalized whitespace and exact visible-boundary output are covered. A separator occupying the last visible position correctly omits the already-read following character.
- At the raw scan cap, diagnostics conservatively show an ellipsis, including extreme leading/trailing whitespace. Exact full-input trim equivalence is intentionally not claimed for that bounded diagnostic case. Renderer content, interaction and DEBUG-off behavior are unchanged. Other trace-target matching is outside this repair.

## Verification integrity

Execution `96356` passed all 38 image-loader and six summary regressions, then exited 101 at official AST validation: 20 passed / three failures. The failures were a 207-line HTML responsibility boundary, literal Japanese test data and a directly specified test background color. Raw: `tmp/failed-decode-html-final.log`; expanded RTK output: `1791323933_cargo_test.log`.

The implementation is split by the cohesive summary responsibility, test text uses Unicode escapes, and the background uses the existing theme bridge. No allowance or quality gate was relaxed. Main also corrected the module's natural filesystem location and restored strict exact-boundary assertions before another compiler execution; these pre-execution checks are not runtime RED evidence.

Corrected verification `93025` exited 0: all 38 image-loader regressions, six summary regressions, 23 official AST checks, strict impacted Clippy, format and diff checks passed. Raw: `tmp/failed-decode-html-corrected-final.log`.

Main's manual coding-rule review then moved the module-facing summary entry from a restricted free function to an associated `HtmlSummary` method, as required by the repository's struct/impl rule. No image-loader source changed. Execution `8595` exited 0: six summary regressions, 23 official AST checks, strict impacted Clippy, format and diff checks passed. Raw: `tmp/html-summary-associated-entry-final.log`. Ordinary signed commits and normal push are not yet complete.

## Ordinary source integration

- Image repair: `29778148bc3d2ab3ff75f00aac2de13710fb96ac`, signature G, ordinary commit execution `45089` exit 0, `tmp/failed-decode-normal-commit.log`.
- Summary repair: `345c095b1c3a295ab0829860059f6c5c21b27baf`, signature G, ordinary commit execution `57116` exit 0, `tmp/bounded-html-normal-commit.log`.
- Latest source commit time: 2026-10-06 22:05:40 UTC / 2026-10-07 07:05:40 JST. English/Japanese changelog timestamps use that actual commit time.

Normal push/current-HEAD review/three-OS CI/coverage, all five distribution assets, actual clean-machine acceptance and publication/postprocessing remain separate unfinished requirements. Earlier HEAD evidence is not substituted. No human application, consent or sibling repository was operated or changed.
