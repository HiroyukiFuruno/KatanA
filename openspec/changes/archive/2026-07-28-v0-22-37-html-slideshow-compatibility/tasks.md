## 1. Release boundary

- [x] 1.1 Keep DOM, CSS, JavaScript, layout, paint, hit-test, and slideshow behavior in KRR's in-process Rust/V8 runtime
- [x] 1.2 Publish and verify KRR `0.4.11` before KatanA registry integration
- [x] 1.3 Resolve KDV `0.3.5` and KRR `0.4.11` from crates.io in the KatanA and headless-harness locks without path/git overrides
- [x] 1.4 Publish and verify the flex-layout correction as KRR `0.4.12`,
  then resolve it from crates.io in both KatanA locks
- [x] 1.5 Publish and verify the pager-alignment correction as KRR `0.4.13`,
  then resolve it from crates.io in both KatanA locks
- [x] 1.6 Publish and verify the dynamic-application compatibility and
  structured main-document diagnostics as KRR `0.4.14`, then resolve it from
  crates.io in both KatanA locks

## 2. KatanA integration

- [x] 2.1 Expose the active HTML frame generation only through test hooks
- [x] 2.2 Add pointer and keyboard headless actions that wait for a newer HTML frame generation instead of using a fixed delay
- [x] 2.3 Pass focused unit tests, formatting, lint, compile, and the complete release contract with no threshold relaxation

## 3. Real slide-deck acceptance

- [x] 3.1 Run the current Google Drive `slides.html` through the headless native KatanA UI at the declared viewport
- [x] 3.2 Prove all 14 slide states and 43 actions, including pointer and keyboard navigation, with no worker stop or JavaScript exception
- [x] 3.3 Generate fresh initial, next-slide, and representative later-slide screenshots from the published registry chain for user inspection
- [x] 3.4 Open a user-entered canonical `file://` HTML URL through the same local-document path as file selection
- [x] 3.5 Render the real local `tmp/index.html` wrapper with its same-directory iframe and relative resources in the in-process Rust/V8 runtime
- [x] 3.6 Generate fresh headless evidence for both the workspace `tmp/index.html` path and the complete oracle wrapper after the local file fixes

## 4. Release readiness

- [x] 4.1 Update the canonical contract and SemVer guard so only published `v0.22.36` -> `v0.22.37` is accepted and withdrawn `v0.29.0` remains rejected
- [x] 4.2 Pass KatanA `v0.22.37` preflight after the CSS-equivalence correction and regenerated local file evidence
- [x] 4.3 Keep Windows CI portable by generating file URLs from native temporary paths and waiting on KDV browser updates instead of polling with a short wall-clock deadline

KatanA commit, push, PR, and release remain prohibited until the fresh screenshots are presented and the user explicitly approves them.
The dirty-tree pre-PR gate is intentionally deferred until that approval permits the release commit.

## 5. User feedback ledger

- [x] 5.1 CSS layout matches Chrome for the real `slides.html`, not only on a reduced fixture
- [x] 5.2 The slideshow advances through click and keyboard input and displays the corresponding updated frame
- [x] 5.3 Chromium, WebView, external browser helpers, and static-renderer fallback are absent from the product path
- [x] 5.4 A local HTML path entered as `file://...` is loaded instead of failing with `Invalid URL: UnsupportedScheme`
- [x] 5.5 A local wrapper document is not left as a blank white page when its same-directory iframe target is available
- [x] 5.6 Compare the same slide state at the same CSS viewport against headless Chrome and enforce bounded differences for element geometry, text metrics, colors, and overflow
- [x] 5.7 Treat browser command saturation as recoverable backpressure: coalesce high-rate pointer move / scroll input without dropping or reordering discrete pointer, keyboard, focus, navigation, and close commands; keep the active frame visible and prove continued frame generation under burst input
- [x] 5.8 Measure the local HTML first-frame path from open request through document load, KDV worker start, KRR session creation, and first frame publication; remove the dominant avoidable delay and add a stable headless latency regression gate
- [x] 5.9 Run the repository-owned `just update` flows for the KDV patch and KatanA release worktrees, resolve all supported dependency updates, and rerun their complete release gates without threshold relaxation or new exclusions
- [x] 5.10 Load same-origin relative iframes from user-entered HTTP/HTTPS
  documents through KRR's resource policy, then prove CSS, JavaScript, iframe
  load, and slideshow input through the native headless KatanA UI.
- [x] 5.11 Do not attach filesystem watchers to virtual `Katana://URL`
  documents; prevent repeated watcher setup warnings and preserve URL-viewer
  responsiveness.
- [x] 5.12 Center anonymous text flex items on both axes inside fixed-size
  circular badges, match Chrome element and glyph geometry with a KRR
  regression contract, and regenerate a full-size native KatanA screenshot of
  slide 7 before release approval.
- [x] 5.13 Center the previous/next glyph layout inside the slideshow's fixed
  circular paging buttons, match Chrome button content geometry with a KRR
  regression contract, and regenerate the full-size native slide 7 evidence
  before release approval.
- [x] 5.14 Reproduce a user-entered dynamic HTTPS application URL through
  native KatanA, distinguish HTTP/browser-challenge rejection from runtime
  JavaScript incompatibility in the UI diagnostic, and define and verify the
  supported browser-equivalence boundary without a Chromium/WebView fallback.

Final dynamic-application evidence from published KDV `0.3.5` and KRR `0.4.14`:
- The public TodoMVC application loads through a user-entered HTTPS URL with
  matching origin and viewport, then pointer input, text entry, and Enter
  create a visible todo item with `261583` changed pixels:
  `tmp/vanillajs-todomvc-dynamic-url-interaction-krr0414-reviewed-final-20260729/`
- `https://chatgpt.com/?no_universal_links=1` reports the server's HTTP 403
  browser-verification challenge, including `Server: cloudflare` and
  `cf-mitigated=challenge`, before CSS or JavaScript starts:
  `tmp/chatgpt-dynamic-url-diagnostic-krr0414-reviewed-final-20260729/01-chatgpt.png`
- User-entered local `file://` documents render through the same browser
  session path, including the complete same-directory iframe wrapper:
  `tmp/v0-22-37-local-file-evidence-kdv035-krr0414-reviewed-final-20260729/`
- Native KatanA completes all 14 Google Drive slide states with a first frame
  in `0.630s`, no worker stop, no JavaScript exception, and at least `482893`
  changed pixels between adjacent slides:
  `tmp/slides-drive-registry-v0-22-37-kdv035-krr0414-reviewed-final-20260729/`
- The complete 81-step browser contract passes HTTP redirect, external
  CSS/JavaScript/image loading, embedded SVG, accordion, button action, text
  input, prevented and allowed links, reload, resize, and frame assertions:
  `tmp/v0-22-37-html-headless-preview-reviewed-final-20260729/`
- The cursor-free slide 7 capture keeps all three badge digits and both pager
  glyphs centered:
  `tmp/slides-drive-pager-center-kdv035-krr0414-strict-final-20260729/slide-07-pager-center.png`
- A 4,097-input burst completes without queue saturation and still advances
  the deck by `4064722` pixels:
  `tmp/slides-drive-input-burst-kdv035-krr0414-strict-final-20260729/`
- Light-theme controls retain a fixed dark background and border; fullscreen
  zoom remains active while right/down/left/up scrolling changes `2664083`,
  `1905548`, `1562410`, and `2563015` pixels:
  `tmp/v0-22-37-light-image-controls-kdv035-krr0414-strict-final-20260729/`
- The headless harness records document path, origin, and generation before an
  open action and accepts success only after that frame identity changes.
- Opening a validated local file cancels pending remote URL responses so a
  stale HTTP result cannot replace the active local document.
- Root and screenshot locks resolve KRR `0.4.14` from crates.io with checksum
  `d68f73aa12cbaa226a7d099943370c87132d51ba4f8772f5df07fdb141149400`.

Final pager evidence from published KDV `0.3.5` and KRR `0.4.13`:
- The KRR button flex-centering contract matches headless Chrome's `40x40`
  button box and glyph range (`x=17`, `y=9.5`, `width=6`, `height=20`), with
  center deltas of `x=0` and `y=-0.5`.
- Native KatanA completed all 14 click-driven slide states at a KRR viewport of
  `1382.0x744.3096`; first frame completed in `0.523s`:
  `tmp/slides-drive-registry-v0-22-37-kdv035-krr0413-final-20260728/`
- The cursor-free pager capture reached slide 7 in six clicks, changed
  `4,083,284` pixels from slide 1, and keeps both paging glyphs visible:
  `tmp/slides-drive-pager-center-kdv035-krr0413-final-20260728/slide-07-pager-center.png`
- Root and screenshot locks resolve KRR `0.4.13` from crates.io with checksum
  `48d8b7511e72f946604d6e9cff93a64e80c789f0a1eb2ade0bdd43018eb305a9`.
- Formatting, Clippy, fixture integration, strict coverage, Linux workspace
  tests, Windows x64 cross-compilation, and the `v0.22.37` preflight pass
  without threshold relaxation or new exclusions.

Final flex-layout evidence from published KDV `0.3.5` and KRR `0.4.12`:
- KRR `anonymous_text_is_centered_inside_a_fixed_size_flex_badge` pins the
  generated text origin to headless Chrome's `x=17.4375` within `1.5px` and
  baseline `y=28.0` within `2.0px` for a `46x46` circular flex badge.
- Native KatanA completed all 14 click-driven slide states at a KRR viewport of
  `1382.0x744.3096`; first frame completed in `0.542s`:
  `tmp/slides-drive-registry-v0-22-37-kdv035-krr0412-final-20260728/`
- Full-size slide 7 shows all three badge digits centered without clipping:
  `tmp/slides-drive-registry-v0-22-37-kdv035-krr0412-final-20260728/slide-07.png`
- Root and screenshot locks resolve KRR `0.4.12` from crates.io with checksum
  `645e38e45c25c423a9af66861201bbbe2073b86c97edf3f3d4eaa8c66560f1e7`.
- Formatting, Clippy, fixture integration, strict coverage, Linux workspace
  tests, and Windows x64 cross-compilation pass without threshold relaxation
  or exclusions.

Previous headless evidence from published KDV `0.3.5` and KRR `0.4.11`:
- KRR/Chrome, same `1230x867` CSS viewport, all 14 slides:
  `/Users/hiroyuki_furuno/works/private/katana-render-runtime-css-v0-4-9/tmp/html-css-probe-v049-final/compare/contact-sheet.png`
- Normalized per-slide pixel MAE is `0.00894216..=0.0229693`; generic KRR contracts separately pin element geometry, mixed Japanese/Latin text metrics, colors, gradients, inline flow, hover, and overflow.
- Native KatanA, all 14 click-driven slide states with at least `482891` changed pixels between adjacent screenshots:
  `tmp/slides-drive-registry-v0-22-37-kdv035-krr0411-final-20260727/contact-sheet.png`
- Native KatanA asserts the KRR viewport exactly matches the HTML display rect (`1382.0x744.3096` logical pixels) before capture.
- Local `file://` missing-child diagnostic and complete same-directory iframe wrapper:
  `tmp/v0-22-37-local-file-evidence-kdv035-krr0411-final-20260727/contact-sheet.png`
- User-entered HTTP URL, same-origin relative iframe, CSS/JavaScript load, and
  click-driven slide transition with `2117174` changed pixels:
  `tmp/v0-22-37-loopback-url-evidence-kdv035-krr0411-final-20260727/contact-sheet.png`
- The unchanged 4,097-input burst request completes without queue saturation or
  worker termination, then ArrowRight advances the deck by `4,064,818` pixels:
  `tmp/slides-drive-input-burst-kdv035-krr0411-final-20260727/contact-sheet.png`
- Final release-profile first-frame runs complete in `0.504s` and `0.513s`,
  both below the explicit `2.0s` regression threshold.
- Light-theme image controls contain `13435` fixed-background pixels at RGB
  `106,106,106`; fullscreen zoom remains active while right/down/left/up
  scrolling changes `2664083`, `1905548`, `1562410`, and `2563015` pixels:
  `tmp/v0-22-37-light-image-controls-final-20260727/contact-sheet.png`
- Root and screenshot `just update` flows completed. Formatting, Clippy, fixture
  integration, strict 100% coverage with no exclusions, AST lint, Linux
  workspace tests, and Windows x64 cross-compilation pass.
