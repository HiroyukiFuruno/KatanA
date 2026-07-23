## Context

KRR parses HTML5 with `html5ever`, evaluates JavaScript in V8, projects the DOM into an in-process layout/paint tree, and uses Taffy for flex/grid flow. The released path nevertheless has two incompatible shortcuts: host-generated events use a plain object instead of the installed `Event` implementation and dispatch only target listeners, while stylesheets are split with string delimiters and filtered by a narrow property allowlist. KatanA also wraps the complete KRR frame in application padding, so the page does not own the full preview viewport. Existing tests verify selected pixels and source values but do not run a realistic URL page through acquisition, subresources, interaction, and navigation.

Constraints are fixed: no Chromium, WebView, external browser helper, static HTML fallback, or browser archive; KRR owns browser semantics, KDV remains an adapter, and KatanA owns main-document acquisition and native surface integration. Coverage remains strict 100% with zero uncovered lines and no exclusions or threshold changes.

## Goals / Non-Goals

**Goals:**

- Use one standards-shaped Event object and one tree-aware dispatch algorithm for host input, synthetic dispatch, inline handlers, and default actions.
- Parse stylesheet rules and declarations with a structured CSS parser, preserve cascade metadata, resolve custom properties, and drive typed computed style plus Taffy layout for a representative modern page.
- Prove final URL origin, relative CSS/JS/image requests, JavaScript DOM mutation, click propagation, form input, and top-level navigation through the complete KatanA -> KDV -> KRR chain.
- Give the HTML page the exact native preview content bounds with no host padding, border, or background visible around its frame.
- Produce deterministic headless screenshots and machine assertions from a realistic page rather than a diagnostic-only fixture.

**Non-Goals:**

- Introducing a platform webview, Chromium-compatible process, network proxy, iframe runtime, or KatanA/KDV HTML semantics.
- Claiming unsupported web-platform APIs without tests. Every compatibility claim added by this change must be represented in the compatibility fixture and contract tests.

## Decisions

1. **Dispatch DOM events over the canonical KRR DOM tree.** `HtmlDocument` will expose the target-to-root element path. Listener registrations will retain capture/once metadata and removal identity. KRR will create the installed V8 `Event`, set `target`, update `currentTarget` and `eventPhase` per phase, honor propagation flags, run inline handlers in order, and only then perform cancelable default actions. This replaces the separate plain-object host event path; adding only `stopPropagation()` to that object would leave capture, bubbling, cancellation, and ordering incorrect.

2. **Use structured CSS tokenization instead of delimiter splitting.** KRR will take a direct dependency on the already-resolved `cssparser` family used by its layout stack. Rule, at-rule, declaration, comments, strings, functions, and `!important` boundaries will be parsed structurally. Existing selector and computed-style types remain KRR-owned but will consume parsed data, preserve source order/specificity/importance, resolve inherited custom properties, and map supported layout values into Taffy. Unsupported declarations remain non-fatal and observable in compatibility diagnostics rather than corrupting adjacent rules.

3. **Grow compatibility from a representative page contract.** A deterministic page will exercise external and inline styles, variables, specificity, descendant/child/attribute selectors, flex, grid, box sizing, overflow, responsive media rules, typography, tables, SVG/image, event capture/bubble/stop, accordion, button mutation, text input, fragment navigation, and another same-origin document. Tests assert DOM state, requested URLs, navigation events, exact viewport dimensions, and stable pixels. This prevents a toy fixture from being treated as browser equivalence.

4. **Keep URL ownership unchanged but test it end to end.** KatanA continues to fetch only the principal document and passes unmodified HTML plus the final response URL. KRR fetches policy-approved subresources relative to that URL. A local HTTP server records every request, performs a redirect, and serves the linked document so final-origin and navigation behavior are verified without internet or CDN dependence.

5. **Remove host chrome at the HTML surface boundary.** The HTML-specific KatanA view will allocate the full available content rectangle directly to the KRR frame. A layout contract test will compare allocated frame bounds with available bounds. Non-HTML preview padding is unaffected.

6. **Release dependency patches in order only after contracts pass.** KRR v0.4.6 is the semantic fix. KDV v0.3.4 is required only if its public dependency floor or adapter contract changes. KatanA v0.22.36 consumes released registry artifacts for final release gates; temporary local integration must not land path/git dependencies.

## Risks / Trade-offs

- **[CSS surface expands faster than renderer support]** -> Keep one explicit compatibility matrix, reject no whole stylesheet because of one unsupported declaration, and add typed computed-style/layout support before marking each matrix row complete.
- **[Event listener mutation during dispatch changes ordering]** -> Snapshot the path and listener candidates while checking removal/once state before each callback, matching DOM dispatch semantics in regression tests.
- **[Network tests become flaky]** -> Use an in-process loopback HTTP server, bounded condition waits, request logs, and no external hosts or fixed sleeps.
- **[KRR patch changes existing static export]** -> Restrict the new CSS mode to the interactive runtime where possible and run the full existing renderer/coverage suite.
- **[Visual evidence hides semantic failures]** -> Require DOM/navigation/request assertions independently of screenshots; screenshots are additional acceptance evidence.
