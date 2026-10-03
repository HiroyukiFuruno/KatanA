# Self-Review: post-v0.22.41 document fidelity regressions

## 2026-09-13 current-diff review

The current review found no P0. Two P1 and two P2 findings were remediated
before commit:

- External Office and supplied `#s15` HTML tests no longer use `#[ignore]`.
  They compile behind a dedicated acceptance feature, while
  `scripts/ci/document-fixture-acceptance.sh` requires both explicit fixture
  inputs and runs the exact tests through `just test-document-fixtures`.
  Ordinary all-feature type checking remains hermetic.
- Explorer projection, XLSX sheet tabs, Markdown rendering, math rendering,
  and paint-metric collection were split by responsibility. Every newly
  created production Rust file is below the 200-line hard limit.
- Screenshot-only overlay inspection is absent from the default public API and
  is exposed only by the explicit `screenshot-test-hooks` feature used by the
  external harness.
- The release-inspector positive fixture now contains valid arm64 and x86_64
  Mach-O slices, macOS 13.0 load commands, and a matching Info.plist. The test
  requires the positive architecture-contract message.

Follow-up checks pass: default and all-feature/all-target KatanA UI type checks,
all-feature/all-target Clippy with warnings denied, all 23 AST lint checks,
both paint-metrics tests, the release-inspector test, the seven architecture
verifier tests, formatting, and `git diff --check`. The complete linked
workspace gate remains pending only on the registry graph's duplicate V8
removal; it was rerun without lowering any threshold and failed at that exact
external boundary.

## 2026-09-12 screenshot execution boundary follow-up

The screenshot CLI no longer accepts and ignores `--binary`. Clap rejects
both argument forms before request loading or fixture creation, and normal
execution explicitly identifies itself as in-process, not packaged acceptance.
The runner usage text now matches that boundary. CI does not pass this option.

Two CLI tests pass in the existing diagnostic KDV candidate graph; restoring
the old ignored argument in that diagnostic copy makes the rejection test fail,
and restoring the fix makes it pass. The initial 34-test diagnostic harness run
passes. Formal published-graph `cargo check --all-targets` passes, but these
results do not prove packaged startup, packaged input, or release readiness.
Task 4.13 remains open for a genuine packaged execution path. Historical suite
counts below are not fresh evidence for the current complete diff.

After integrating the XLSX fixture follow-up, the complete diagnostic harness
suite passes 35 tests and formal all-target Clippy passes with warnings denied.
The fixture clicks the Notes accessibility label and checks selected index 1
with a bounded condition, not a blind ten-second delay. This does not yet prove
the actual XLSX scenario or bottom-rail geometry; task 4.14 remains open.
No source changes were committed or published in this follow-up.

## No issues after remediation

- Workspace Clippy passes for all targets with warnings denied.
- The repository AST lint passes all 23 checks.
- The KatanA UI suite passes 1,051 tests with 6 pre-existing explicit
  external/manual ignores; no ignore or acceptance relaxation was added.
- The external screenshot harness passes all 29 unit tests.
- No added `todo!`, `unimplemented!`, `dbg!`, empty assertion, lint-disable,
  visual snapshot, fixed browser wait, or lowered threshold was found.
- Added source and comments use English; OpenSpec and evidence Markdown pass
  the repository's targeted language/format checks.

## Findings remediated during review

The first AST-lint review failed and was not treated as acceptable technical
debt. Four files exceeded the 200-line responsibility boundary and four new
public free functions violated the struct-and-impl rule. The correction:

- Split document render inspection from render lifecycle work.
- Split HTML surface state/metrics from preview-pane orchestration.
- Split HTML harness hooks from general application test hooks.
- Split Explorer virtualization from workspace/projection orchestration.
- Moved workspace revision, debug logging, startup heartbeat, and visible-row
  range behavior onto their owning types.
- Further split preview-sidebar rendering into layout and auxiliary-control
  responsibilities and replaced repeated string matching with a typed panel
  target.
- Replaced incremental `zip -r` publication with a fresh `ditto --keepParent`
  archive after moving cached stale DMG/ZIP artifacts aside, so removed bundle
  entries cannot survive from an Actions cache.

Type checking, Clippy, AST lint, screenshot-harness tests, and the complete UI
suite all passed after these corrections.

## Remaining gate boundary

The review conclusion is **FAIL for release readiness**, not for the KatanA
changes above. The unchanged full link/release contract still correctly rejects
the two published V8 versions introduced by KDV 0.5.5 and KRR 0.4.19. Full
verification must be rerun after published KDV adoption; this blocker is
recorded in `v8-dependency-link-gate.md` and was not bypassed.

## KRR 0.4.19 diagram reference candidate follow-up

The controls-off candidate regeneration exposed a KatanA texture-lifetime bug:
hiding controls also removed the only retained `TextureHandle`. The review
rejected both a lint suppression and an additional boolean parameter. A typed
`RasterizedInteraction::Hidden/Visible` boundary now keeps texture ownership
separate from control visibility. All call sites are inside `preview_pane` and
were updated; function visibility was narrowed accordingly.

The added test asserts retained texture ownership, while the real screenshot
runner proves rendered diagram pixels and no `Missing texture` warning. No
snapshot expectation, score threshold, ignore, or fixed harness timeout was
added. Focused format, all-target Clippy, 173 preview-pane tests, screenshot
harness settings, two-run artifact determinism, strict OpenSpec validation,
and diff checks pass. Conclusion: **PASS for this follow-up diff**; final source
commit remains subject to the repository's post-verification approval rule.

The follow-up `sample.md` export generator uses the existing typed
`export_png` harness step and a fixed light `KatanaLight` reference theme. It
does not replace a reference from KDV output or modify KDV. Two KatanA runs
resolved exact registry KDV 0.5.5 and KRR 0.4.19 and produced identical
dimensions and SHA-256. The initially explored dark output was rejected before
evidence recording because it did not match KDV's white export-reference theme.

KDV's independent v0.5.6/KRR 0.4.19 validation does not accept either image:
the export scores 89/95 and the diagrams crop scores 56/95. Accordingly, these
files remain deterministic candidates rather than canonical references. The
diagram score path currently lays out at 1280x2400 even though the KatanA crop
represents 1187x2225 logical content pixels; that capture-contract mismatch is
under reconciliation. No threshold, ignore, or unverified artifact adoption is
part of this KatanA diff.

After KDV adopted the corrected crop geometry, the diagram score improved to
88/95. The remaining approximately 8% diagram-bbox difference is an owner-layer
version mismatch: KatanA uses registry KUC 0.3.3, while KDV Storybook uses the
Git-tagged Storybook 0.3.0. KUC Issue #35 now tracks the neutral publishable
boundary. KatanA does not add a path/git override or weaken KDV's neutral UI
boundary to hide this difference.
