## Why

The browser-equivalent HTML viewer still failed on a real self-contained slide deck: viewport-relative sizing, positioned controls, gradients, nested inline text, and multilingual wrapping produced a visibly broken composition, while pointer and keyboard navigation could capture a stale frame. A release must prove the actual Google Drive `slides.html` document through the native KatanA -> KDV -> KRR path rather than through a reduced static sample.

## What Changes

- Consume the published KRR `0.4.9` Rust/V8 runtime from crates.io in both KatanA and its headless harness.
- Wait for the HTML frame generation associated with pointer and keyboard input before capturing the next slideshow state.
- Prove all 14 slides and 43 scripted actions from the real `slides.html` document, including click and keyboard navigation.
- Keep Chromium, WebView, external browser processes, browser archives, and static HTML fallback out of the product path.
- Allow only the adjacent KatanA release from published `v0.22.36` to `v0.22.37`; keep withdrawn `v0.29.0` rejected.

## Capabilities

### Modified Capabilities

- `html-file-preview`: Require real slide-deck layout, input-driven repaint, registry-only dependency, and release-evidence contracts.

## Impact

- KatanA HTML surface test hooks and headless screenshot harness.
- KatanA dependency locks, release contracts, version metadata, and changelogs.
- No HTML parser, CSS engine, JavaScript interpreter, or hit-test implementation is added to KatanA or KDV.
