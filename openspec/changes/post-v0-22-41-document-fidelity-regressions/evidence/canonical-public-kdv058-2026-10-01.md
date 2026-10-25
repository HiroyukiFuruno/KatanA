# Current public graph canonical capture

## Provenance

- Source HEAD: `7b5224d4e6c452446329d7d8f2e4b3c4778e086c`.
- Registry graph: KDV `=0.5.8`, KUC `0.4.0`, KRR `=0.4.21`, V8 singleton `152.2.0`.
- Root lock SHA256: `71e11503a40c1ef725da7efa71fc48cf51d1b14d541854fa8e6bbb58bbe1ac0b`.
- Screenshot lock SHA256: `6c9fbf51d7d10c3e6306bba4ea5524bd8cc4eee223bf2e68c949aac9eea2bb05`.
- Runner SHA256: `3c77e9e1dd4e068681e3eabccd293436e4fea4cdcf14cf2240f1e234e3079876`.
- Generator: `scripts/screenshot/generate-sample-diagrams-reference.sh`, exit 0.
- Request: `scripts/screenshot/examples/sample-diagrams-canonical-capture.json`, unchanged.
- Diagram fixture SHA256: `88f5cfae620fa721cd0a53765402833dc6e705dfc27c6cd1808143530e043dd3`.
- Execution mode: in-process; packaged binary NOT tested.

## Candidate artifacts

Local root: `tmp/canonical-kdv058-egui0362-diagrams-7b5224d4/`.

| Artifact | SHA256 |
| --- | --- |
| `sample-diagrams-top-candidate.png` | `c6b6326f76374ea2712ca634b877ece7162acf7ea949ec1c1c4ee23b0fc31577` |
| `raw/sample-diagrams-full.png` | `f77baa764224dcc653ed0288263dab16c292d8a353933d4278e6343c8ed67bf1` |
| `raw/sample-diagrams-preview-geometry.json` | `8ed27489f487873a91fbf6e6af5a3d54ce5436c7d1a3d8fe324ecd11a2993ac3` |

Geometry: pixels-per-point 2; full PNG 2565×4774; physical crop
2374×4450 at (88, 268); normalized candidate 1280×2400.
Viewport: (44, 134), 1187×2225 logical points; scroll 0; font size 14.
Completed frame and geometry frame both 1223. All hover/selection/image/
diagram/code controls counts are zero, with pointer outside content.

## Acceptance boundary

Full sample.md export repeated twice with unchanged fixture/request/source.
Both generators exit 0; both PNGs are 1280×19282 RGBA and byte-identical
(`cmp` exit 0). SHA256:
`2980205a94a2cfcedde6d22f6f3032d8d794ce156f86e91efd93908c57a36168`.
Roots: `tmp/canonical-kdv058-egui0362-export-7b5224d4-run1/` and `-run2/`.
Fixture SHA256: `489360a81d60af20d67f8ea47e251732a194983431ea1ab261627490cc9009e1`.
Export request SHA256: `f3eb890e32ff375e63fa334a0b5f8e100cab70a40bf4f2dde06ea18b444aaefa`.
Diagram request SHA256: `d05e6851066d0d9b3789bcc2d358b3fc172eda2df6431f6f97120cdecd6fbfbb`.

The existing sample.md line 65 contains a malformed SVG namespace attribute
(`xmlns=%22<http://www.w3.org/2000/svg%22>`). The actual generator reports the
SVG parse error; repeatability is not proof that this input renders correctly.
Do not alter the canonical fixture/baseline merely to suppress this warning.
Existing CSS/filter warnings also remain in the raw run output.

Actual `test-canonical-interaction-contract.sh` succeeds: controls-on capture
is rejected for a rendered code-copy overlay, and typography/diagram captures
require all controls/hover/selection counts zero and matching completed frame.
Evidence root: `tmp/canonical-kdv058-egui0362-interaction-7b5224d4/`.

Candidate generation and clean-interaction geometry succeeded. Reference assets
were NOT changed. Independent KDV current-public-graph category scores and
explicit adoption evidence remain pending. Actual supplied HTML, Office fidelity, and packaged acceptance
remain separate unfinished release gates.
