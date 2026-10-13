# HTML sticky TOC ownership evidence

## Fixture and expected behavior

- Fixture: `/Users/hiroyuki_furuno/Desktop/KatanA-不具合/要件定義書_v1.0.html#s15`
- The fixture defines `.app` as a flex container and `.sidebar` as `position: sticky; top: 0; height: 100vh`.
- User-observed Chrome behavior: the left TOC remains visible after navigating or scrolling to `#s15`.

## KatanA/KRR reproduction

- The real KatanA source-intake path was exercised through a KRR HTML session at the supplied fragment.
- First frame: 17,210 ms.
- Fragment scroll position: `scroll_y=3048.824`; content height: `9539`.
- Dark sidebar color (`#0f172a`) ratio in the first frame: `0.0000`; the sidebar has scrolled out of the viewport.
- The external-fixture acceptance is ignored by default and fails while this defect remains, so normal test runs are not made flaky by a desktop-only fixture.

## Owner boundary

- KatanA forwards the source and fragment into KRR and displays the returned frame.
- KRR 0.4.17 parses `static`, `relative`, `absolute`, and `fixed`, but has no `sticky` `CssPosition` variant or layout branch.
- Consequently `position: sticky` is retained as the previous/default position and rendered in normal flow. The missing primitive is owned by KRR, not KatanA's preview layout.

## Chrome differential (2026-08-29)

- Chromium 1.62.1, viewport `1440x900`, local `file://` fixture.
- At `scrollY=0`, `.sidebar` computes to `position=sticky`, `top=0`, `bottom=900`, `height=900`.
- After a 1,200 px wheel-equivalent scroll, `scrollY=1232` and the sidebar geometry remains exactly `top=0`, `bottom=900`, `height=900`.
- Chrome's `IntersectionObserver` changes the active TOC item to `href=#s13`, `text=機能要件一覧` at the measured position.
- Screenshots: ignored local artifacts `output/playwright/html-chrome-s15-top.png` and `output/playwright/html-chrome-s15-scrolled.png`.

## Current KatanA differential

- KatanA now keeps the sidebar visible after the same 1,200 px scroll, so the original sticky-disappearance defect is fixed by the published KRR correction below.
- KatanA still leaves `変更履歴` active while Chrome changes the active section. KRR reports `event_targets=0` for this fixture, proving that the remaining JavaScript/observer behavior is not projected.
- KatanA also renders the page at a materially larger text/table scale than Chrome at comparable viewport width. This remains owner-layer task 3.2 and is tracked by [KRR issue #73](https://github.com/HiroyukiFuruno/katana-render-runtime/issues/73).
- Current trace: script execution `7.570 ms`; first layout `680.730 ms`; first SVG rasterization `389.577 ms`; first frame `1,070.391 ms`. Subsequent layouts are approximately `81-83 ms`, while rasterization remains `371-724 ms`, so JavaScript execution is not the dominant delay.

## Supplied-page interaction regression

`scripts/screenshot/examples/supplied-html-interaction-regression.json` now
exercises the actual supplied file through the KatanA host rather than a reduced
HTML fixture:

- The KRR viewport exactly matches the `1162.5 x 741.8884` KatanA display rect.
- A 1,200 px downward wheel input settles at `scroll_y=1195.58` of
  `max_scroll=9296.11`; the reverse input settles back at `0.00`.
- The wheel result changes `2,166,048` screenshot pixels.
- A real TOC pointer click advances the HTML frame and changes `1,634,797`
  pixels, proving KatanA action dispatch is live.
- Opening the exact user URL ending in `#s15` produces HTML generation 2 and
  lands on the `1.5 未確定事項` section while the full-height sidebar remains
  visible.
- The active TOC color still remains on `変更履歴`; this is the unresolved KRR
  IntersectionObserver portion tracked by issue #73, not a KatanA input or
  scrolling failure.

Cold timing also separates the normal product path from diagnostic logging.
With both `RUST_LOG` and `DEBUG` unset, the first frame completed in `1.047 s`
and the direct `#s15` navigation frame completed in `1.486 s`. Runs with
`RUST_LOG=error` took `1.184-2.043 s`. The earlier `5.883 s` result was measured
under the shell's ambient `RUST_LOG=warn` setting and emitted hundreds of
identical Helvetica Neue to Arial Unicode MS fallback warnings; it is not the
release-default GUI configuration. KRR issue #74 tracks warning
aggregation/sampling and a cold font-database timing gate for broad diagnostic
logging. The interaction test keeps a 10 s functional timeout so it does not
disguise that performance work as an input failure.

The post-self-review release rebuild repeated the same exact supplied-file
request with `RUST_LOG` and `DEBUG` unset: first frame `1.293 s`, direct `#s15`
frame `1.560 s`, viewport match unchanged, wheel settle `1195.58/9296.11`, and
both screenshot-change assertions passed.

## Local KRR correction

- Published owner artifact: `katana-render-runtime` v0.4.18 ([PR #67](https://github.com/HiroyukiFuruno/katana-render-runtime/pull/67), [GitHub Release](https://github.com/HiroyukiFuruno/katana-render-runtime/releases/tag/v0.4.18), crates.io registry).
- `CssPosition::Sticky` is parsed distinctly; it is not aliased to `fixed`.
- Top and bottom scrollport offsets preserve the element's normal-flow contribution, and the available containing-block boundary clamps sticky movement.
- A self-contained `#s15` flex-sidebar regression passes and asserts at least 16,000 `#0f172a` sidebar pixels after fragment scrolling.
- The supplied `要件定義書_v1.0.html#s15` external-fixture regression passes and asserts at least 100,000 sidebar pixels after fragment scrolling.
- The supplied debug fixture required 368.10 seconds for its first frame. A CPU sample attributes the dominant work to table-cell wrapping and repeated text-width measurement, so HTML evaluation performance remains task 3.2 rather than being hidden by the sticky fix.
- Verification: KRR library coverage run 875 passed/1 ignored; sticky 7 passed/1 ignored; the complete `just check` gate passed with 996 workspace tests/1 ignored; the new `layout_sticky.rs` has 100% region/function/line coverage; the strict workspace line-coverage gate passed at 100% with zero uncovered lines; AST responsibility gate, formatting, strict all-target/all-feature clippy, TypeScript, Biome, runtime bundles, checksums, package verification, and crates.io dry-run passed.
- KatanA resolves `katana-render-runtime =0.4.19` from crates.io with no path/git override. The supplied host acceptance keeps the sticky sidebar visible after scrolling.
- KatanA's complete `just check` passed after the registry update, including strict clippy, the host test suites, the Linux container workspace tests, the Windows cross-target check, formatting, and the AST responsibility gate.
- KRR publication and its KatanA registry integration are complete. Other owner-layer publications and the packaged multi-file acceptance remain tasks 5.1 and 5.2.
