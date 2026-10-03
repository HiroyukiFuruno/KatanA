# Render artifact binding checkpoint

## Current review finding

PR #346 current HEAD ba1bc215 has P1 comment 4171200192, thread
PRRT_kwDORm09y86oivbN. Comparing hash strings and receipt rectangles does not
prove that measurements originate in accessible independent reference and
packaged-main rendered outputs. The repair must read actual artifact bytes,
verify digests, bind metrics to those renders and match the measured artifact
to the recorded input/run/target/main identity. Moving self-declared rectangles
into another unverified JSON is insufficient.

## Independent source-reference progress

The existing KDV owner measured the first exported page of all six original
Office inputs without changing the originals or fixed references. Its private
manifest SHA-256 is
3708b43b8cf5b083323961d7d6bd2feabe86fb8f678dd747975d5ad982d27284;
producer-environment SHA-256 is
36bceeb8bd777bee8c04507297bfdf6b1fafaaec0bde81cb94ad1c7abfad1fbd.
The owner verified input/record/PDF/PNG hashes, decode dimensions and finite
named element geometry. Main read the handoff checkpoint, not a new independent
rerun of that producer. Private text and images are not committed here.

The original default headless reference had missing Japanese glyphs. A separate
native macOS LibreOffice backend improved that reference, and the owner retained
both runs. This reference-backend problem is not a KatanA product fix or proof of
full font equivalence.

These are natural source-print canvas measurements, not the 1280x900 host
viewport. Only the first print page is measured; continuous Excel grids and all
slides/sheets are not covered. Candidate geometry, missing-element count,
tolerances and independent scores remain unevaluated. Representative-fixture
tolerances must not be transferred to these originals. The next producer work
must map actual packaged document viewport/offset/scale and preserve run and
render identity; no acceptance receipt is fabricated from this checkpoint.

## Public dependency boundary

Live GitHub Release and sparse-index checks still report KRR 0.4.22 and KDV
0.5.9 non-yanked. The required KRR performance repair is not published. Root
keeps exact public registry versions; KRR owner is running its complete gate.
Latest KatanA PR remains Draft, with macOS/Windows test/build running and the
other eight checks successful. Public release, clean-machine acceptance and
post-release cleanup remain incomplete.

## Repair and verification boundary

Main rejected the delegated JSON-only draft and implemented two-stage binding:
tracked comparison contract -> reference metrics JSON -> PNG bytes, and
receipt -> measured metrics JSON -> PNG bytes. Both metrics JSON digests are
computed from accessible files. Each render digest is computed from actual PNG
bytes; the PNG signature/IHDR dimensions must match the logical viewport and
explicit pixel ratio. This is byte/header integrity, not a full PNG decoder or
visual correctness score.

The measured manifest's run/input/target/main/sidecar identity must match the
recorded packaged run. The packaged run independently declares render path,
digest, pixel ratio and metrics path/digest, all matched to the measured files.
HTML additionally verifies the packaged main/sidecar against the declared
target artifact. Geometry, viewport, active state, navigation and Office
missing-element count must match the loaded measurements. Validator helpers
do not inject hidden fields into the receipt.

Three targeted regressions against unchanged ba1bc215 produced 21 assertion
failures because bad artifacts/metrics/run output were accepted. The production
source was not replaced for this negative control; the old module was loaded
from Git into a separate Python object. Synthetic PNGs and metrics are generated
only in disposable unit-test directories and are not actual acceptance evidence.
Logical 1280x900 with physical 2560x1800 at ratio 2 is covered without changing
the agreed viewport or any comparison tolerance.

Final `python3 -B -m unittest` execution passed all 54 tests (38 acceptance,
16 release gate). Direct script execution also passed both suites before the
file-relative import repair. Diff whitespace checks passed. Main self-review
confirmed actual files are read, traversal/symlinks/non-files are rejected,
contract files remain inside source identity, metrics are compared with loaded
artifacts and measured output is bound on both manifest and run sides. No
source-hash exclusion, tolerance, performance limit or score was relaxed.

Independent review was reconciled with current files: the reported contract
directory exclusion and old directory-writer error were absent/fixed in current
source. Its standard-unittest helper import finding was reproduced and fixed
with file-relative module loading; final combined unittest success confirms it.
Self-review conclusion: PASS for the P1 validator repair, not product fidelity.
Actual independently produced artifact/measurement receipts are still required;
this byte-binding gate is not a producer attestation or cryptographic provenance
system and does not derive semantic element rectangles from PNG pixels.
