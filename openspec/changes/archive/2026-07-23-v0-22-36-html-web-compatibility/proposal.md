## Why

The released HTML viewer proves only a narrow fixture and does not satisfy its browser-equivalent contract: normal event handlers fail on `stopPropagation()`, common CSS is silently discarded, the page viewport is inset by application chrome, and URL documents are not verified with their relative resources and navigation. These gaps must be corrected at the KRR browser-semantics boundary and proven through the complete KatanA -> KDV -> KRR path.

## What Changes

- Implement DOM event propagation in KRR, including capture, target, bubble, cancellation, immediate propagation stop, listener options, handler properties, and default-action ordering.
- Replace the ad-hoc CSS token splitting and narrow property allowlist with structured parsing and extend computed style/layout/paint for the representative web-page contract.
- Verify HTTP/HTTPS main-document loading, final redirect origin, relative stylesheet/script/image loading, same-origin link navigation, and visible failure diagnostics through KatanA.
- Make the HTML browser viewport fill the preview content bounds without KatanA padding, border, or background leaking around the page.
- Add deterministic local-HTTP compatibility fixtures with CSS, JavaScript, accordion, form input, event propagation, resource loading, and navigation, plus headless screenshots and machine assertions.
- Keep KRR as the Rust/V8 browser-semantics owner, KDV as a session adapter, and KatanA as the document-acquisition/UI shell. Chromium, WebView, external browser processes, and static-renderer fallback remain forbidden.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `html-file-preview`: Strengthen the existing browser-equivalent contract with concrete DOM Event, CSS, URL-resource, viewport, and release-evidence requirements.

## Impact

- KRR HTML runtime, CSS parser/cascade, computed style, layout/paint, subresource loader, and interaction tests.
- KDV browser-session adapter tests and dependency floor if a new KRR patch release is required.
- KatanA URL acquisition/navigation tests, HTML preview layout, headless screenshot harness, release gates, and dependency floors.
- Patch releases are expected in dependency order: KRR v0.4.6, KDV v0.3.4 if its public dependency/adapter contract changes, then KatanA v0.22.36 after user visual acceptance.
