# CI Verification

- Pull request: [#332](https://github.com/HiroyukiFuruno/KatanA/pull/332)
- CI run: [32475852947](https://github.com/HiroyukiFuruno/KatanA/actions/runs/32475852947)
- Release Readiness: [32475852948](https://github.com/HiroyukiFuruno/KatanA/actions/runs/32475852948)

## Required Checks

- Release Readiness passed.
- Dependency Supply Chain, CodeQL, and macOS/Linux/Windows lint jobs passed.
- `Test and Build (macos-latest)` passed, including strict coverage and `multi-format-headless-macOS`.
- `Test and Build (ubuntu-latest)` passed, including `multi-format-headless-Linux`.
- `Test and Build (windows-latest)` passed, including `multi-format-headless-Windows`.

## External Hyperlink PPTX Evidence

Each platform artifact contains `07-pptx-slide-1.png` and `08-pptx-slide-2.png` from the 37-step multi-format acceptance scenario. The generated fixture carries the standard external hyperlink relationship before KatanA opens it.

Manual inspection of each platform's first PPTX frame confirmed:

- KatanA presents a `1/2` PPTX page instead of a KDV diagnostic surface.
- The rendered slide includes positioned shapes, an embedded image, text, and Japanese glyphs.
- No external hyperlink target was opened or fetched by the KatanA host.

These artifacts provide review evidence; the acceptance assertions remain the deterministic verification of the document-surface behavior.
