# KRR 0.4.19 sample export reference candidate

## Tracking and source

- KatanA issue: <https://github.com/HiroyukiFuruno/KatanA/issues/338>
- KatanA branch: `fix/post-v0.22.41-document-fidelity`
- Source base commit: `af55949c4ee7ddad414ed93fa360a67e9464acf4`
- Fixture origin commit: `16c868c014c2a3f5af96d8b718978af8a5e6abb9`
- Final source commit: pending the required post-verification commit approval
- KDV dependency: `katana-document-viewer = "=0.5.5"`
- KDV Cargo.lock source: crates.io registry
- KDV Cargo.lock checksum: `a68026488f72aaf159272fa16380934b08847904732c355f87c4c3ba77cac6f6`
- KRR dependency: `katana-render-runtime = "=0.4.19"`
- KRR Cargo.lock source: crates.io registry
- KRR Cargo.lock checksum: `57a1ef0b7309a6c2e11f78541ba07a0c7b0a88aa02a15ceb6004c7cdc325653d`
- KDV/KRR path or git override: none

The repository has unrelated local patches for vendored UI and MathJax crates,
but neither KDV nor KRR appears in `[patch.crates-io]`. Both exact packages in
the resolved lockfile have registry source and checksum entries.

The final source commit is deliberately left pending. The generator and
artifact are verified working-tree changes on top of the source base commit,
so the base commit alone must not be presented as artifact provenance.

## Artifact contract

- Manifest: `scripts/screenshot/examples/sample-canonical-export-reference.json`
- Generator: `scripts/screenshot/generate-sample-export-reference.sh`
- Source fixture: `assets/fixtures/sample.md`
- Export theme: light / `KatanaLight`, matching the KDV export-reference tokens
- Overlay diagram controls: disabled
- Candidate output: `assets/reference/katana/export_png/sample.png`
- Dimensions: `1280x19067`
- SHA-256: `2dddc1c54b07e3f1132ea0e269d2106e074b3864b1faafb734110939b3afe66d`

The KDV owner session independently isolated the old `1280x19097` reference
to KRR 0.4.16/V8 150.0.0 and the new `1280x19067` candidate to KRR 0.4.19/V8
152.2.0. The KatanA registry-only generation above reproduces the new height.
The candidate is not yet accepted as canonical: KDV 0.5.6/KRR 0.4.19 reports
89/95 (`average=100`, `content=89`) against it. The exact dimensions and
background agree, so reference replacement remains held while KDV isolates the
content-only difference. The 95-point threshold is unchanged.

## Reproduction and determinism

```bash
rtk proxy scripts/screenshot/generate-sample-export-reference.sh
```

Two independent executions produced the same `1280x19067` dimensions and the
same SHA-256
`2dddc1c54b07e3f1132ea0e269d2106e074b3864b1faafb734110939b3afe66d`.
The first 1,200 rows were inspected after generation and contain the expected
white export background, dark text, centered HTML content, link color, rule,
and badge row rather than an empty or low-contrast surface.
