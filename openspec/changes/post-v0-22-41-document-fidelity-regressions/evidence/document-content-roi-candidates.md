# Document-content ROI candidates (2026-09-13)

## Result and scope

### Fresh worktree contract rerun on 2026-09-13

The screenshot harness now uses the same vendored `egui-winit` 0.36.1 input
adapter as the KatanA workspace. Its refreshed independent lock retains the
known-compatible public KUC 0.3.3 graph until KDV 0.5.6 is published. All 38
harness tests pass.

A fresh `katana-screenshot` build then ran
`scripts/screenshot/test-canonical-interaction-contract.sh` with an absolute
runner, source root, and new output root. It exited 0 after rejecting the real
controls-on capture and accepting both same-frame controls-off typography and
diagram captures. This closes the stale-runner/invalid-invocation gap, but it is
still in-process evidence; registry-only and packaged acceptance remain open.

### Controls-off follow-up: verified

`scripts/screenshot/test-canonical-interaction-contract.sh` completed with exit 0:
the real controls-on capture was rejected with 12 code-copy and 15 diagram-control
renders, and both controls-off captures passed. Logs are under
`target/typography-host.tyoRkw/interaction-contract-stable/`.
Observation is reset before a real UI step, completed with that step's frame number,
then recorded alongside its rasterized PNG. Both positive captures have a positive
Markdown-render witness, matching completed/captured frame numbers, pointer (1,1)
outside content, null active editor line, and zero hover/selection/control counters.
This verifies the interaction contract, not visual-score or release acceptance.

The v2 artifacts and machine-readable provenance are at
`target/typography-host.tyoRkw/output-canonical-provenance-overlay-v2/`.
Full PNG SHA-256: typography `2171d6f263be8abda85529bfaec426f51d546999fb4677ec9ab878ca4e0b4776`;
diagrams `14af024425e1e3327afecc94f990c2f0c4d4b40a2e73a95a0a099609f669cb27`.
Normalized content hashes remain identical to those listed below.
Runner SHA-256: `3367b4720c2de6929493685035d5d5d2a3dfa092416d19b35f849ae4fd94278c`.
Snapshot inventory SHA-256: `ac712a4abe033bff00a78fab63adca0c1b2c656490d4a9a535ebeb84f9a72f7d`.

Formal observer unit tests passed 3/3; diagnostic request tests passed 9/9;
formal screenshot all-target Clippy passed with warnings denied. Shell syntax and
scoped diff checks passed. All three requests are JSON-equivalent between formal
worktree and snapshot. The eleven capture Rust files were compared: remaining
differences are closure braces, comments and JSON macro formatting, not behavior.
The snapshot alone adapts a PDF test fixture to the candidate KDV API.
It still uses a diagnostic path dependency on KDV candidate `3b4ecb1`, registry
KRR 0.4.19 and KUC 0.3.7. Final registry-only recapture after KUC 0.3.11 and KDV
publication remains required. Existing reference PNGs and threshold 95 are unchanged.

### Historical first capture (not clean-controls acceptance)

Follow-up requirement: KDV's existing `score_visual_interaction()` disables hover,
selection, image, diagram and code controls. Its `assert_no_overlay_controls()`
requires zero copy-code, copy-source, fullscreen, zoom-in, reset-view and media
overlay host actions. The original candidate manifests do not prove this full
contract. KatanA's Markdown viewer independently enables code copy outside
slideshow, so absence of a pointer action does not prove absence of that icon.
Same-frame interaction records and rejection tests are being added before the
next candidate handoff. The older four PNG hashes below remain historical
diagnostic evidence, not clean-controls acceptance.

Both fixtures were captured through the same diagnostic KatanA screenshot runner.
The `record_preview_geometry` step saves the full PNG and geometry from the same
render frame and rejects incorrect logical size, scroll position, or clipped
physical content. Existing reference PNGs and the 95-point threshold are unchanged.

Output root, relative to this implementation worktree:
`target/typography-host.tyoRkw/output-canonical-provenance/`.
`provenance.json` records source, binary, input, configuration and output identity.

| Contract | Both captures |
| --- | --- |
| Logical host viewport | 1282.5 x 2387 |
| Full physical PNG | 2565 x 4774 |
| Logical document content | x=44, y=134, width=1187, height=2225 |
| Pixels per point | 2 |
| Physical content crop | 2374x4450+88+268 |
| Scroll / font | 0 / 14 logical pixels |
| Normalization | ImageMagick Box, 1280x2400, stripped PNG32 RGBA |

`sample.md` retains Light/KatanaLight; `sample_diagrams.md` retains Dark/KatanaDark.
The manifests disable diagram/image controls and contain no pointer or selection
actions. However, pointer position, active editor line and every possible overlay
are not mechanically recorded. Do not claim full controls/overlay acceptance from
these settings alone. KDV/KUC must validate the candidates against the existing
document-content contract before any canonical adoption.

## Files and SHA-256

- `typography/sample-typography-full.png`: `b59f69aeb9b92f31169182756e62ceb4fa947dd99b97f6e9d3d2502f32ac2a3b`
- `typography/sample-typography-document-content.png`: `7183c95d24e7910dfb088f837cb8b37c8faa98c4f01ad51441c1287009ee91dc`
- `diagrams/sample-diagrams-full.png`: `e6c77cbffd71377f5d7cc1b67c5536cd2c78db1f079274c5b8c0d020883aa1ae`
- `diagrams/sample-diagrams-document-content.png`: `31fc692ccbe6a803242768ae3a5fd8df7346f8ec0307fb353dd09ea54d99ecda`
- Both directories contain a corresponding `*-preview-geometry.json`.
- Runner: `target/typography-host.tyoRkw/katana/scripts/screenshot/target/debug/katana-screenshot`, SHA `d1070e3e0582cc7d6b5fed80616355b5c25013c0af387a1d05d7272f3d0f900d`.

## Reproduction and validation

Use the two `scripts/screenshot/examples/*-canonical-capture.json` requests with
the recorded runner. From each recorded full PNG, normalize without overwriting
any reference:

```sh
magick INPUT-full.png -crop 2374x4450+88+268 +repage -filter Box \
  -resize '1280x2400!' -strip PNG32:OUTPUT-document-content.png
```

The main agent re-hashed all four PNGs, verified both geometry JSON files and
confirmed both normalized images are 1280x2400 sRGBA. Current formal screenshot
all-target Clippy with `-D warnings` passed. The capture worker reports request
tests 9/9 and fmt passing. These are in-process diagnostic artifacts, not packaged
startup, published-dependency acceptance, or release proof.

The snapshot has no independent `.git`; Git commands there resolve the parent
worktree. Parent HEAD/diff identity must therefore not be represented as a complete
snapshot source identity. See the corrected source-tree hash scope in the manifest.
