# Current-source canonical sample diagrams candidate

## Provenance correction: not accepted as a fixed-source reference

After capture, a manifest/lock update was detected at 2026-10-14 00:36 JST: zip 9.0, uuid 1.28 and compatible transitive updates. This occurred after the clean input identity record (00:30 JST) but before the actual capture (00:42 JST). The runner build was already active. The recorded old input hashes therefore do not prove an immutable source/dependency snapshot for the entire build. The successful geometry and output hashes remain real, but the fixed-clean-HEAD claim below is superseded. This PNG is retained as a diagnostic candidate only; it must not be adopted as the Issue #338 authoritative reference. The KDV recipient has been sent this correction. Regeneration is pending after the dependency API migration and Canvas repair are verified and formally integrated.

Changed root lock: `31e2f449f83ee4d4efbd5ad1a52209d817db9fd643ae7257cd8805127a5ee7bd`; changed screenshot lock: `8fd21cb4b180c862a02203337ac3301de921ef3214ee8b20f2be0b208f3189f4`. Original inputs, actual images, logs and the failed compile are preserved rather than overwritten or relabelled as success.

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/338

The official generator completed with exit 0 (session 81921). It was started from public HEAD `ad4adb87dbf2a74d1e81c796803d2ab624f6a36a`, but the manifest/lock changes during its build invalidate an immutable-source attribution. It ran the regular screenshot runner in-process, built the explicit release Office worker and runner, captured the actual app preview, checked same-frame clean interaction geometry, and normalized the preview crop to 1280×2400. Existing reference assets were not overwritten.

Approved registry dependencies remain KDV 0.5.12, KRR 0.4.23 and KUC 0.4.1 in both locks. The older version mentioned in the original Issue is not used to downgrade the current release graph.

## Reproduction

```sh
rtk proxy scripts/screenshot/generate-sample-diagrams-reference.sh \
  /Users/hiroyuki_furuno/works/private/katana-post-v0.22.41-document-fidelity/tmp/issue338-ad4adb87-capture-2026-10-14
```

The generator requires a new absolute output directory. It rejects overwriting an existing output or writing a candidate into `assets/reference`.

## Recorded identities (inputs were recorded before the dependency change)

| Input/output | SHA-256 |
| --- | --- |
| Root Cargo.lock | `5b092e46ed6109d3946298f348cdfcc72fe388e528905eef8d9c562c0950f903` |
| Screenshot Cargo.lock | `f0f9723d996124861673ea8457a4e8a196534e6564deae8e2527a96333618b12` |
| assets/fixtures/sample_diagrams.md | `88f5cfae620fa721cd0a53765402833dc6e705dfc27c6cd1808143530e043dd3` |
| Capture request | `d05e6851066d0d9b3789bcc2d358b3fc172eda2df6431f6f97120cdecd6fbfbb` |
| run.sh | `3b6255776cc65964f8b75da62fa53018d0a00992c0751828879f42841a6e0bad` |
| Generator | `2f30777c5873bb84cb9bc1fbfae926fa6d88bb2ab9df80579e6da1d0b68417bb` |
| Release Office worker | `92a5481430e506d8d8d11e983e7bcb5417a32de36cbbe5ae97a2f3852f2f4e6b` |
| Screenshot runner | `bcbaea9b5e48ef29681e7dfdadec3b2fbd96bbbdf5292912c71e83d2c8cce4c7` |
| Raw full PNG | `526e248f8626e9f6c1b1cbb01f504ec38812c2213ae539abd3d684a0d6e8b3fb` |
| Geometry JSON | `8ed27489f487873a91fbf6e6af5a3d54ce5436c7d1a3d8fe324ecd11a2993ac3` |
| Normalized candidate PNG | `2f998a05bd1663d62a0378bc83f43c8608c80373ee624bb6756e9909aabb8c5b` |

## Geometry and boundaries

Raw image: 2565×4774. Physical crop: `{x:88,y:268,width:2374,height:4450}`. Logical content viewport: 1187×2225, scroll 0. UI pass and completed interaction frame both 1223, Markdown sections 17. Pointer is outside content; editor line is null; hover, selection and control counts are all zero.

Candidate path: `tmp/issue338-ad4adb87-capture-2026-10-14/sample-diagrams-top-candidate.png`. Raw image and geometry are in the adjacent `raw/` directory. Input/output hashes and execution output are preserved in `tmp/issue338-ad4adb87-{input-identity,output-identity,capture}-2026-10-14.log`.

The actual candidate was visually inspected after geometry checks. Flow, sequence, class and state sections are present. The state label `CommandNotFound` is visibly clipped; unsupported CSS/SVG warnings remain in the raw log. These limitations are not hidden by replacing a downstream baseline to obtain a passing score.

This is a candidate with `execution_mode=in_process` and `packaged_binary_tested=false`, not an adopted reference, Chrome parity proof, independent 95-point score or packaged/clean-machine acceptance. A later source or renderer version requires its own identity and acceptance; this capture is not relabelled as a future HEAD result. Consumer handoff and formal evidence publication are tracked separately.
