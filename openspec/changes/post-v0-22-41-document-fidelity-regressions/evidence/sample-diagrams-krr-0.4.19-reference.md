# KRR 0.4.19 sample diagrams reference candidate

## 2026-09-13 provenance audit correction

The historical fixed crop below must not be treated as a verified document-only
rectangle. The recorded raw screenshot is no longer present at its documented
path in either the implementation worktree or the saved root. The candidate
PNG still matches `890c69378bdadd78c1b11f0558dadf9988483fb2662a2cd24792ec48fadb66ef`,
but that alone cannot validate its ROI. KDV reports header/tab contamination;
the separate real-host typography capture measures content origin `(44,134)`
logical, not the old crop origin `(32,101)`. These are different capture
configurations, so do not infer an exact corrected crop by subtracting them.
Task 4.15 tracks new same-runner full screenshots and measured content ROI
manifests. Existing reference pixels and the 95-point gate remain unchanged.

## Tracking and source

- KatanA issue: <https://github.com/HiroyukiFuruno/KatanA/issues/338>
- KatanA branch: `fix/post-v0.22.41-document-fidelity`
- Source base commit: `af55949c4ee7ddad414ed93fa360a67e9464acf4`
- Fixture origin commit: `1f7cd6cdcc02b8aa81351bf2a3875bfd333c9332`
- Final source commit: pending the required post-verification commit approval
- KRR dependency: `katana-render-runtime = "=0.4.19"`
- Cargo.lock source: crates.io registry
- Cargo.lock checksum: `57a1ef0b7309a6c2e11f78541ba07a0c7b0a88aa02a15ceb6004c7cdc325653d`

The source commit is deliberately not overstated: the candidate runner, texture
lifetime correction, and reference artifact are currently verified working-tree
changes on top of the source base commit. KDV must use the final source commit
recorded here after KatanA commit approval, not treat the base commit alone as
the artifact provenance.

## Artifact contract

- Manifest: `scripts/screenshot/examples/sample-diagrams-canonical-reference.json`
- Source fixture: `assets/fixtures/sample_diagrams.md`
- Viewport: `1280x2400` logical pixels at the macOS 2x device scale
- Theme: dark / `KatanaDark`
- Diagram controls: disabled
- Raw screenshot: `target/canonical-reference/sample-diagrams-krr-0.4.19/sample-diagrams-full.png`
- Raw screenshot dimensions: `2560x4800`
- Physical crop: `2374x4450+64+202`
- Normalization: ImageMagick Box filter to `1280x2400`, with metadata stripped
- Candidate output: `assets/reference/katana/preview_crops/sample-diagrams-top.png`
- SHA-256: `890c69378bdadd78c1b11f0558dadf9988483fb2662a2cd24792ec48fadb66ef`

## Reproduction

```bash
rtk proxy scripts/screenshot/generate-sample-diagrams-reference.sh
```

The script runs the KatanA screenshot harness and then applies the exact crop,
Box resize, and metadata stripping contract. Two independent runs produced an
identical raw screenshot SHA-256
`44bd217d5d7a78e7732bcc4b917159376adf79a1fa34bc07597b365205cff021`
and an identical candidate SHA-256
`890c69378bdadd78c1b11f0558dadf9988483fb2662a2cd24792ec48fadb66ef`.

This candidate is not yet a canonical KDV reference. The KatanA window is
1280x2400 logical pixels at 2x, but the captured preview content is the physical
rectangle `2374x4450+64+202`, or 1187x2225 logical pixels. The current KDV score
path instead lays out the scene at 1280x2400 logical pixels and rasterizes it at
`2374 / 1280`, then downsamples to 1280x2400. With KDV 0.5.6 and KRR 0.4.19 this
scores 56/95 for content. Responsive diagrams therefore receive different
layout widths, so canonical adoption is held until the score harness models the
same content viewport without lowering the threshold.

KDV subsequently applied the 1187x2225 logical / 2x / 2374x4450 / 1280x2400
normalization contract, improving the diagram score to 88/95 but not passing
the unchanged threshold. The remaining major diagram bounding boxes are about
8% smaller in KDV because KatanA resolves registry `katana-ui-core 0.3.3`, while
the KDV Storybook renderer still resolves the Git-tagged
`katana-ui-core-storybook 0.3.0`. KUC Issue
<https://github.com/HiroyukiFuruno/katana-ui-core/issues/35> tracks a publishable
neutral Storybook boundary; KDV correctly declined to absorb a `publish=false`
Storybook crate that directly resolves eframe/egui. Canonical adoption remains
held until that owner-layer boundary is published and the 95-point test passes.

## KatanA defect found while regenerating

The controls-off path passed `None` as the diagram viewer state. This dropped
the `egui::TextureHandle` before the renderer consumed the paint command and
produced blank diagrams with `Missing texture` warnings. KatanA now retains the
viewer state and texture while independently suppressing fullscreen/zoom/pan
controls. The focused regression test
`show_rasterized_keeps_texture_when_controls_are_hidden` passes, and the
candidate run contains the diagram pixels without `Missing texture` warnings.

## Verification

- `cargo test -p katana-ui --lib --locked preview_pane -- --test-threads=1`:
  173 passed, 3 pre-existing ignored.
- `cargo clippy -p katana-ui --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --check -p katana-ui`: passed.
- Screenshot harness diagram-control setting test: passed.
- Generator shell syntax and manifest JSON parse: passed.
- Strict OpenSpec validation and `git diff --check`: passed.
