# Current public graph capture and resource cycle

## Immutable executable provenance

- Built production source: `f7931084a52cf953951ebe9a9908c8ea0f431943`.
- Source was frozen until both binaries and the canonical capture completed.
- Registry KDV `0.5.8`, KRR `0.4.21`, KUC `0.4.0`.
- Root lock SHA256: `71e11503a40c1ef725da7efa71fc48cf51d1b14d541854fa8e6bbb58bbe1ac0b`.
- Screenshot lock SHA256: `6c9fbf51d7d10c3e6306bba4ea5524bd8cc4eee223bf2e68c949aac9eea2bb05`.
- Runner SHA256: `ca7732438c99525011ac4379c017df346331cced2025e6aee0b3c6ba1bf1987f`.
- Office worker SHA256: `8f919b20a27562652fd7550ecdb0029e9e4a8c5287f46d53a945e7d30056ea8d`.
- Mode: in-process; packaged main binary not tested.

## Canonical candidate

The unchanged generator and request completed with exit 0. Output root:
`tmp/canonical-kdv058-f7931084-current-worker/`.

- Candidate SHA256: `c6b6326f76374ea2712ca634b877ece7162acf7ea949ec1c1c4ee23b0fc31577`.
- Full PNG SHA256: `f77baa764224dcc653ed0288263dab16c292d8a353933d4278e6343c8ed67bf1`.
- Geometry SHA256: `8ed27489f487873a91fbf6e6af5a3d54ce5436c7d1a3d8fe324ecd11a2993ac3`.
- Geometry and completed frame: 1223; physical crop 2374x4450 at (88,268), normalized 1280x2400.
- All hover, selection, code/image/diagram overlay counts zero; pointer outside content; scroll zero.

Reference assets were not overwritten. Candidate hashes match the earlier
candidate, but that is provenance, not an independent fidelity score or an
adoption decision. KDV #58 remains open.

## Actual mixed HTML/XLSX cycle

The freshly built executable was run directly without another build, with
`DEBUG=false` and the exact Office worker above. Request:
`scripts/screenshot/examples/document-resource-cycle.json`, SHA256
`9665f66820ae44673d1d46ae64c900716de5d00371958895b2c68bf991a8ef60`.
Output root: `tmp/resource-cycle-f7931084/`; operation results were returned
in the runner stdout/tool receipt (no screenshot or JSON output was requested).
Execution completed all nine steps and quit with exit 0.

| Point | RSS KiB | UI frame | HTML/document observed |
| --- | ---: | ---: | --- |
| Cold idle | 103456 | 106 | 0/0 |
| Warm idle after first cycle | 204304 | 223 | 1/1 |
| After ten further cycles | 233520 | 293 | 11/11 |

First HTML frame 0.027s, first XLSX frame 2.575s. Warm HTML frames were
0.024-0.031s and XLSX frames 0.047-0.053s. XLSX identity was Grid, 2 sheets.
Warm RSS delta was 29216 KiB, below the unchanged 65536 KiB threshold;
cold delta 100848 KiB was below the unchanged 196608 KiB threshold.
All recorded preview, HTML/document surface, Office worker, frame, texture,
and cache counters were zero at each idle point. Both runtime assertions passed.

These counters are host observations, not an OS descendant identity proof.
This simple HTML fixture is not the supplied #s15 document. This run does not
prove PPTX cycles, Office fidelity, clean-machine packaged RSS, or resolution
of the newly identified blocking intake worker accumulation. The executable
predates that forthcoming repair; its acceptance must be repeated afterward.

## Additional actual PPTX cycle

The same frozen binaries also completed a real HTML/PPTX 1+10 cycle with exit0.
Request was retained as `scripts/screenshot/examples/document-pptx-resource-cycle.json`
after execution; only its display name was generalized. It uses the same
baseline RSS and timeout thresholds as the existing XLSX request, with the
existing representative PPTX identity (2 slides, Page) instead of Grid.
Retained request SHA256 before formatting:
`a63693281aeb444b892ab05fbde4a0f03366bee73123a2b165d5e5e314dd9cb0`.
The retained request after the normal Biome formatter has SHA256
`c6fe9a682d8879efb28a9aaf8384267819c4ed6b03f5192384b476d850b453dc`;
formatting did not change its parsed operations or thresholds.
The run used `tmp/pptx-resource-cycle-f7931084.json` and output root
`tmp/pptx-resource-cycle-f7931084/`; stdout contains all nine completed steps.

| Point | RSS KiB | UI frame | HTML/document observed |
| --- | ---: | ---: | --- |
| Cold idle | 103408 | 106 | 0/0 |
| Warm idle after first cycle | 185712 | 248 | 1/1 |
| After ten further cycles | 193936 | 801 | 11/11 |

First PPTX frame 2.785s; warm frames 0.969-1.099s. Warm RSS delta 8224 KiB
and cold delta 82304 KiB both pass the unchanged thresholds. All idle internal
resource counters zero, both assertions passed, normal runner quit exit0.
This extends the in-process cycle evidence to the small representative PPTX,
not the supplied large Office files or packaged main executable. The future
intake repair still requires a fresh binary and repeat acceptance.
