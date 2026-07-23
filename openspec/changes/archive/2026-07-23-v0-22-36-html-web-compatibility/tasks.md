## 1. Reproduction and compatibility contract

- [x] 1.1 Add a KRR regression that reproduces `e.stopPropagation is not a function` through host click dispatch and proves capture/target/bubble order
- [x] 1.2 Add a realistic KRR fixture for external/inline CSS, variables, specificity, `!important`, flex, grid, box sizing, overflow, media rules, tables, SVG/image, and JavaScript recascade
- [x] 1.3 Add a loopback HTTP contract that records redirect, relative stylesheet/script/image, and linked-document requests without external network dependencies

## 2. KRR v0.4.6 DOM Event runtime

- [x] 2.1 Replace plain host event objects with the installed V8 `Event` implementation and expose the canonical DOM ancestor path
- [x] 2.2 Implement capture, target, bubble, inline-handler, cancellation, immediate stop, listener removal, capture, and once semantics with correct ordering
- [x] 2.3 Prove default-action and navigation behavior after `preventDefault()` / propagation handling and preserve actionable script location diagnostics

## 3. KRR v0.4.6 CSS and layout runtime

- [x] 3.1 Replace brace/semicolon/colon splitting with structured CSS rule and declaration parsing, including safe recovery from unsupported declarations
- [x] 3.2 Preserve source order, specificity, inline priority, `!important`, inherited custom properties, and `var()` resolution in interactive cascade
- [x] 3.3 Implement the fixture's computed-style/layout properties through typed values and Taffy, including box sizing, overflow, responsive media rules, and required typography
- [x] 3.4 Recompute cascade/layout/paint after DOM class, attribute, inline style, or custom-property mutation

## 4. KRR and KDV verification

- [x] 4.1 Pass KRR focused event, CSS, resource, navigation, and frame tests plus the full suite
- [x] 4.2 Pass KRR strict 100% line coverage with zero uncovered lines, AST lint, package verify, and publish dry-run without exclusions or threshold changes
- [x] 4.3 Confirm KDV `0.3.3` needs no adapter or dependency-floor change and accepts the KRR `0.4.6` patch through its existing registry requirement

## 5. KatanA v0.22.36 integration

- [x] 5.1 Remove HTML-only host padding/frame and add an exact preview-bounds contract without changing non-HTML preview spacing
- [x] 5.2 Add loopback URL acquisition tests for redirect final origin, relative resources, runtime-confirmed navigation, active-tab history, and navigation queue ordering
- [x] 5.3 Integrate the fixed registry dependency chain for final gates without landing path/git dependencies or browser binaries
- [x] 5.4 Update the SemVer guard so only published v0.22.35 -> v0.22.36 is accepted and v0.29.0 remains rejected

## 6. Acceptance and release readiness

- [x] 6.1 Run KatanA full checks, strict 100% coverage, OpenSpec strict validation, package/release gates, and self-review
- [x] 6.2 Generate headless before/after screenshots and machine assertions for CSS layout, event propagation, accordion, button, input, resources, URL navigation, resize, scroll, and exact viewport bounds
- [x] 6.3 Present the realistic rendered evidence for user visual confirmation before KatanA commit, push, PR, or release

## 7. User feedback ledger

- [x] 7.1 `e.stopPropagation is not a function` を単一メソッド追加ではなく DOM Event 伝播契約として根本解決する
- [x] 7.2 CSS がまともに適用されない原因である簡易文字列 parser / 限定 cascade / layout を構造化実装へ置換する
- [x] 7.3 HTML 表示外の KatanA 固有 padding / border / background をなくし page viewport を全面表示する
- [x] 7.4 URL 指定の principal document、redirect 後 origin、relative CSS/JS/image、リンク遷移を実際の UI 経路で証明する
- [x] 7.5 Chromium / WebView / 外部 browser process を導入せず Rust/V8 の KRR ownership を維持する
