# Current-source supplied Office memory observations

Source: `9b0226ba`, frozen throughout these runs. All observations below are
in-process harness executions, not packaged-app acceptance or full Office fidelity.
Private requests refer to the supplied files from ignored `tmp/`; document contents
and private absolute source paths are not added to the public history.

## Executables and unchanged acceptance limits

- Runner SHA-256: `a7afc8df165284507bde7e48285359add89b4994cc4f8ae2a273803f0a0f7f7c`.
- Office worker SHA-256: `8f919b20a27562652fd7550ecdb0029e9e4a8c5287f46d53a945e7d30056ea8d`.
- First document frame: 15 seconds; close: 5 seconds.
- Cold RSS increment: 196608 KiB; warm increment: 65536 KiB.
- Thresholds, assertions, coverage exclusions and product safety limits were not relaxed.

## Actual supplied inputs

The supplied 82 MiB XLSX reached a real Grid first frame in 5.949 seconds.
Cold RSS was 103728 KiB; open RSS 257792 KiB; close RSS 279248 KiB.
The increment 175520 KiB passed the existing cold limit (exit 0).
A second fresh process reached its frame in 4.581 seconds and closed at
291808 KiB from 104016 KiB (187792 KiB increment, exit 0).
200 ms OS sampling recorded host peak 291808 KiB and direct Office child
peak 94640 KiB over 31 samples. These are sampled peaks, not continuous maxima.

Five other supplied files were opened and closed sequentially in one fresh process:

| Input class | First frame seconds | Frame identity | Open RSS KiB |
| --- | ---: | --- | ---: |
| Small XLSX | 0.941 | Grid, 1/5 sheets | 195312 |
| Order XLSX | 0.100 | Grid, 1/1 sheets | 211136 |
| Proposal PPTX | 4.402 | Page, 1/26 slides | 240112 |
| Large PPTX | 2.020 | Page, 1/9 slides | 396336 |
| OIDC PPTX | 1.457 | Page, 1/11 slides | 420144 |

Cold RSS was 105152 KiB. After all closes it was 420096 KiB.
The 314944 KiB increment exceeded 196608 KiB: **FAIL, exit 1**.
All existing internal counters were zero, which does not explain this RSS retention.
Real first frames prove neither workbook/slide content fidelity nor distribution readiness.

## Unmodified representative cycles

The official HTML/XLSX 1+10-cycle request passed at this source (exit 0):
cold 103888 KiB, warm 253472 KiB, final 263376 KiB.
The warm increment was 9904 KiB, within 65536 KiB.

The official HTML/PPTX 1+10-cycle request passed (exit 0):
cold 103872 KiB, warm 183456 KiB, final 193600 KiB.
Initial Page frame took 2.847 seconds; repeated frames took 1.000–1.327 seconds.
The warm increment was 10144 KiB, within 65536 KiB.
One direct-run setup initially omitted the explicit worker path and failed before
the first frame; the valid rerun set `KATANA_KDV_OFFICE_WORKER` to the same hashed
worker used for supplied-file acceptance. That setup error is not a product result.

A single supplied large PPTX with representative HTML also failed the cold limit:
104080 to 331040 KiB, increment 226960 KiB (exit 1); real Page frame 2.859 seconds.
Separate warm-only diagnostics do not erase that cold acceptance failure.
The separate large-PPTX/HTML warm-only diagnostic also failed: 331024 to
975056 KiB after ten additional cycles (644032 KiB increment, limit 65536,
exit 1). This is repeated-input growth, not merely a five-unique-file observation.

An A/B attempt initialized WGPU before its cold baseline and rendered after each
close in the five-file sequence. It stopped at the proposal PPTX: its real first
frame took 21.154 seconds, exceeding the unchanged 15-second deadline (exit 1).
No final RSS comparison was obtained. The failure is retained; deadlines are not
extended to complete this diagnostic.

An HTML-free, repeated-large-PPTX diagnostic also failed its unchanged warm
limit: 272240 to 918800 KiB after ten additional closes (646560 KiB increment).
Rendering after every close with WGPU initialized before the baseline did not
eliminate growth: 350448 to 1176240 KiB (825792 KiB increment, exit 1).
Renderer initialization alone is therefore not an adequate explanation.

## OS allocation classification

A separate run sampled the same runner process with readonly `vmmap -summary`
at two RSS thresholds. No memory pressure/reclamation API was invoked.

| Measurement | Early sample | Later sample |
| --- | ---: | ---: |
| Physical footprint | 274.4 MiB | 726.4 MiB |
| Malloc Large active resident | 88.1 MiB | 88.1 MiB |
| Malloc Large empty, dirty | 164.9 MiB | 605.4 MiB |
| Total malloc bytes allocated | 118.3 MiB | 118.3 MiB |

The increasing footprint is predominantly freed-but-resident allocator memory
at these sample times, not proportional growth in live malloc allocations.
The samples include an open/render stage, not a synchronized session-close
boundary. They do not prove every session/counter is released, explain exactly
which allocation path populates the empty regions, or establish that the packaged
app has the same behavior. The RSS regression remains failed.
The public macOS SDK `malloc/malloc.h:520–526` defines best-effort allocator
pressure relief and its returned released-byte count. No product use or solution
is claimed without an actual measured experiment and platform verification.

## Proven measurement gap, not a proven product leak

`KatanaApp::preview_resource_counts_for_test()` only visits current `tab_previews`.
Removing the preview therefore removes it from that count before its detached
`katana-document` worker necessarily finishes `DocumentSession::close()`.
The harness close loop checks this count and Office subprocesses, not live KDV
sessions, retained artifacts/page/cell caches, or completion of the document worker.
PPTX conversion subprocess termination is normal while host PDF rendering is active.

Published KDV 0.5.8 already exposes `DocumentSession::resource_snapshot()` with
eight process-wide resource counters. A root-only diagnostic projection plus a
worker-lifetime guard can observe these without editing KDV or changing its API.
These independently loaded atomics are not a transactional snapshot; polling for
zero is appropriate only in the isolated single-app harness, not parallel UI tests.

The private supplied-file runs did not initialize the harness WGPU renderer via
Screenshot. Pending GPU uploads therefore are not established as their cause.
PDF temporary PNG/RGBA allocations, allocator high-water marks and worker close
timing remain hypotheses. No source owner for the residual RSS is confirmed yet.

## Independent allocator experiments

An isolated optimized Rust executable allocated, touched and dropped ten buffers
of 64 through 73 MiB. RSS rose from 1840 to 703552 KiB despite each Vec being
dropped before measurement. SDK best-effort `malloc_zone_pressure_relief(NULL, 0)`
reported zero released bytes and no immediate RSS reduction (1785 microseconds).
Repeating with inherited `MallocNanoZone=0` removed produced the same pattern:
1856 to 703568 KiB, relief zero. This environment flag alone does not explain it.

A control with ten identical 64 MiB buffers instead plateaued at 67520 KiB
(cold 1840 KiB), both with and without that flag. The changing-size allocation
experiment demonstrates allocator retention without a live Vec leak, but does
not establish which allocations differ across repeated PPTX opens.
These are isolated diagnostic executables, not product fixes or acceptance runs.
No allocator API, environment setting, RSS limit or product allocator was changed.

## KatanA bounded-reader reproduction

`source_io::read_limited` starts with an empty Vec and wraps File in `Read::take`
before `read_to_end`. An isolated optimized executable reproduced this exact read
shape using the same supplied 40852621-byte PPTX, without KDV, GUI or PDF rendering.
Across eleven reads/drops, capacity was always 67108864 bytes and idle RSS rose
1840 to 445376 KiB. A second fresh process that reserved the opened file length
within the same limit before the same bounded read plateaued at 41872 KiB.

This demonstrates a KatanA-owned allocation-growth/retention path. It is not yet
proof that a preallocation change resolves the complete app regression. File
growth/shrink, nonregular streams, read errors and the limit+1 oversize check
must remain supported. The preceding frozen-source coverage run completed with
exit 0 and strict document-surface coverage 100%. The capacity regression for a
generated 3145729-byte regular file fails on the old reader because its capacity
is 4194304 bytes, above the 3145730-byte sentinel. The bounded preallocation
implementation and subsequent product acceptance are in progress; thresholds
and allocator settings remain unchanged.

## Strengthened lifecycle acceptance

The strengthened lifecycle runner (working-tree diagnostic, based on `9b0226ba`,
SHA-256 `8b9e0639fb765db0b50923e73595b2abd39ed0d3a5fbc370fbd62dfb8c13bcda`)
completed eleven opens of the same large PPTX. Every idle snapshot reported zero
document workers and all eight KDV resource counters zero. Warm RSS still rose
from 268896 to 915344 KiB (646448 KiB increment): **FAIL, exit 1**.
First frame was 2.672 seconds; repeated frames were 2.139–2.179 seconds.
Waiting for actual document-resource release alone does not solve this regression.

An identical-size isolated allocator control using a fresh joined thread for each
64 MiB allocation also plateaued at 67552 KiB. Fresh threads alone therefore do
not reproduce the approximately 64 MiB-per-document growth.

The strengthened official representative PPTX cycle run stopped during cycle five
with missing Office response, while free disk reached 119 MiB. Earlier cycles
produced first frames, but the overall request failed (exit 1). Disk pressure is
an observed diagnostic constraint, not a proved explanation for the protocol error.
Retain this failed run and repeat after available space is stable.

## Remaining acceptance

- Bounded-reader working-tree runner SHA-256
  `5165e3438239e19db3ee20aec28c42b9f3269c2084c163de96d6c23f22fa274d`
  completed the same HTML-free large-PPTX eleven-open request with exit 0.
  Warm RSS was 301424 to 306960 KiB, a 5536 KiB increment; every idle worker and
  all eight KDV counters were zero. This improves the former 646448 KiB increase.
- The same binary still failed the mixed HTML/PPTX cold budget with exit 1:
  104320 to 358992 KiB, delta 254672 above 196608. The five supplied Office files
  also failed with exit 1: 105008 to 420512 KiB, delta 315504 above 196608.
  These failed runs are not replaced by the passing repeated-input diagnostic.
- The supplied 82 MB XLSX request passed with the same binary: first frame
  3.462 seconds, RSS 103824 to 293936 KiB, delta 190112 within 196608, exit 0.
  All idle worker and KDV counters were zero. This is not all-workbook fidelity.
- Both official representative resource-cycle requests passed unchanged:
  `document-pptx-resource-cycle.json` and `document-resource-cycle.json`, exit 0.
- The KDV responsibility and unmeasured candidates are tracked separately in
  [Issue #59](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/59).
  Root source bytes move without a deep clone. Five read/drop-only inputs in a
  fresh process rose 1840 to 66608 KiB, delta 64768; this does not reproduce the
  full unique-Office 315504 KiB increase. PDF temporary buffers and allocator
  retention remain candidates, not established causes or completed repairs.
- Observe document worker completion and all published KDV resource counters.
- Repeat the same supplied-file sequence with strengthened release observation.
- Separate repeated-input high-water behavior from unique-input retention.
- Verify actual packaged-process RSS and human fresh-profile startup consent.
- Do not publish or mark memory regression resolved on these intermediate results.

## Additional font-stage diagnostic

The same bounded-reader runner executed the five supplied Office inputs with
`DEBUG=true`; no budget or input was changed. The result remained **FAIL, exit 1**:
cold RSS 105504 to 423488 KiB, delta 317984 above the 196608 KiB budget.
This diagnostic does not replace the normal-DEBUG acceptance above.

The first two XLSX lookups completed in 82124 and 82248 microseconds, both with
zero resolved faces. No root font lookup was recorded for the three PPTX opens.
RSS was 233024 KiB at the third input and 412256 KiB at the fourth.
These observations narrow the measured stages; completion alone does not exclude
allocator retention from the earlier temporary font reads.

The existing document-worker count did not include font lookup threads. Current
PR review comment4159589365 identifies that observation gap; lifecycle accounting
is being extended, and the former zero-worker results do not prove those threads
had finished. OS allocator, TLS and GPU release remain separate from owned counts.
The raw ignored log is `tmp/office-font-phase-rss-diagnostic-run-2026-10-02.log`.
The KDV owner received the source, request and measurements through Issue #59 and
the existing task; no sibling implementation or new worktree was created here.

## Reacceptance after font-worker accounting repair

The actual in-process runner was rebuilt with locked public dependencies at
KatanA source `e69bdfa6046e76d2d8b3ddf479b66d08503fd532`. The release build exited0;
the executable was copied to an immutable ignored path before execution.

- Runner SHA256: `e9ec42318abc34cfb2689301f160edfe6a3c1810e88bf6a6de3ad3c5d2bf9168`.
- Explicit release-worker SHA256: `be0283a71ce5e41cca487b46bd9818b4a6ef0dce2f9921907c81286595ca5a2d`.
- Unchanged request SHA256: `f58b929fab7430afc4eb5893f49bccd0d7a7efd6a8cf2fddc29339c7a3641d0c`.

With `DEBUG=false`, the same five supplied Office documents produced five
document frames. Final close observed zero previews, surfaces, workers, frames,
textures, cache entries and all eight KDV resource counters. Font threads now
participate in the document-worker count until owned work actually finishes.
The unchanged cold budget still **FAILS, exit1**: 105024 to422352KiB, delta317328
above196608KiB. UI frames advanced76 to439. No process from this exact run remained
in the final process check. The ignored raw log is
`tmp/font-lifecycle-office-sequence-e69bdfa6-2026-10-02.log`.

This is strengthened resource-observation acceptance, not a memory fix or a
packaged-binary test. The explicit sidecar differs from the earlier8f919b20
sidecar, so these runs are not claimed as a source-only controlled performance
A/B. The original failed evidence and unchanged threshold remain intact.

The KDV owner's independent fresh-process PDF and original-lifetime measurements
are recorded in [Issue59](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/59#issuecomment-5939908326).
Those measurements distinguish freed-but-resident large allocations from live
owners and show a diagnostic early-source-drop benefit, but do not constitute a
product repair or passing KatanA acceptance. No sibling implementation is made
here while the existing owner requests a usable checkout.
