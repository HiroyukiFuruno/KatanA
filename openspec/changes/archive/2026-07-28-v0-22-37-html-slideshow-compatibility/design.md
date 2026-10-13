## Context

KRR owns the in-process Rust/V8 DOM, CSS, JavaScript, layout, paint, hit-test, and input semantics. KDV transports session commands and frames. KatanA fetches the principal document and presents complete KRR frame pixels. The Google Drive `slides.html` document is the acceptance input because it combines viewport-relative geometry, positioned layers, gradients, multilingual text, `Element.closest()`, pointer input, and keyboard navigation in one real page.

## Decisions

1. **Keep compatibility fixes in KRR.** KatanA does not interpret the slide deck. It consumes published KRR `0.4.9`, which provides the required generic CSS, DOM, font, iframe, and repaint behavior without fixture-specific selectors.
2. **Wait on frame generation, not elapsed time.** The headless harness records the active HTML frame generation before an input and waits for a newer generation. This makes the screenshot correspond to the JavaScript-updated document rather than to a timing guess.
3. **Exercise both input paths.** The acceptance run advances the real deck by rendered pointer targets and keyboard events, verifies every resulting slide frame, and fails on a worker stop or JavaScript exception.
4. **Separate independent browser comparison from the product.** A system browser may be used only as an external visual oracle. No browser executable, helper, process, or archive enters the KatanA/KDV/KRR product or release path.
5. **Gate publication on fresh evidence.** Mechanical checks and headless screenshots complete before KatanA commit, push, PR, or release. Those Git operations require explicit user acceptance of the presented evidence.

## Risks / Trade-offs

- **A frame can advance without the expected slide state.** Machine assertions compare each new screenshot with the prior one and verify the complete 14-slide sequence, rather than treating generation alone as success.
- **The Drive file can change.** The run records the exact source file, dimensions, action request, and produced screenshots together so the evidence remains auditable.
- **Font differences can hide layout regressions.** KRR `0.4.9` uses one resolved face for measurement and paint; comparison is performed at the exact KRR viewport dimensions.
