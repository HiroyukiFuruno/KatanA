# Supplied HTML acceptance with public KRR 0.4.22

## Result

FAIL: the original document produced its first frame in 36.776 seconds and
navigated to `#s15` in 30.437 seconds, but normal close exceeded the unchanged
five-second operation budget. The external supervisor terminated the run at
75 seconds; maximum sampled CPU was 84%. Process cleanup subsequently found no
owned residual process. Cleanup is not evidence of successful normal close.

The viewport, final origin and screenshot steps succeeded. Earlier KRR 0.4.21
acceptance did not produce a first frame within the original 60-second budget.
This is partial progress, not document-fidelity or release completion.

## Immutable identity and scope

- Mode: `in_process_host`, not the packaged application.
- Source HEAD: `f3d6dadbf9f25ccb910652c5e1c3e322f604284b` with locally modified
  dependency manifests/locks; this is not a fully committed dependency snapshot.
- Original input: 53,818 bytes, SHA256
  `c02d2d7a2420e4e15e3d98a044a310c67bc75fa858c95c4867b9c3f5d7aca012`.
- Request SHA256:
  `dc0fddd4d6b23884afd4d681b047dd4058e7a75b7ed035ba4f0d0ac8c03321e2`.
- Viewport: 1280 x 900; fragment: `s15`.
- Runner SHA256:
  `4a6a8aa46afb6f124f9f98aeed6621ad0c5f3f2b17809ec375a0345771e32487`.
- Office sidecar SHA256:
  `731126734efcd0ca15a9f045871df40ed24f7b0d564a455957b13280f9a6ca54`.
- Runner lock SHA256:
  `da1921b9ebabebc2c77934bc56666c6c2879bcd0278e44945c51dab550ef06cb`.
- Registry versions: KRR 0.4.22, KDV 0.5.8, KUC 0.4.0; calloop 0.14.4.
  Calloop was updated to 0.14.5 only after this run finished. Later tests on that
  graph do not retroactively change this acceptance result.
- Actual KRR archive SHA256 matches the non-yanked sparse-index checksum:
  `545ed2ebc3a8ccd8e39c588f7e2ff4add90c58d110bfb1d6fe9792a7865a3465`.

Local raw evidence remains in `target/supplied-html-krr-0.4.22/`:
`runner.log`, `evidence.txt`, `cpu-samples.tsv`, `process-cleanup.json`,
`process-receipt.json`, `process-samples.jsonl`, `input-identity.json`, and
`artifact-identity.txt`. Private document contents are not published here.

## Close-path diagnosis and handoff

Root `app/doc_close.rs` invokes `cleanup_closed_tab_previews`; the retain in
`app/action/mod.rs` drops the removed HTML surface. Published KDV 0.5.8
`browser_session.rs` implements adapter Drop by calling close, which requests
command-queue close and synchronously joins its worker. That worker dispatches
runtime operations before receiving the next command. An expensive outstanding
runtime operation can therefore make this synchronous close path wait.

The harness calls `trigger_action(ForceCloseDocument)` before entering its local
idle-deadline loop. The independent supervisor still detects the over-budget
action. No sampled blocked stack was collected, so the exact blocking stage and
root cause are not yet proven. Do not classify this as uniquely KRR-owned or
claim that a detached worker, larger timeout, or zero post-kill resources fixes
the product lifecycle contract.

Measured evidence and the root/KDV call path were sent to the existing KRR and
KDV owner chats. No sibling implementation or additional worktree was created.
Normal close, remaining rendering latency, Office RSS, original quality scores,
packaged acceptance, full changed-dependency gates and publication remain open.

## Recheck after the compatible calloop update

The unchanged request on calloop 0.14.5 again failed normal close: first frame
28.327 seconds, fragment action 26.757 seconds, external timeout at elapsed
63 seconds, maximum sampled CPU 81.7%. Input and executable hashes remained
identical; the runner lock SHA256 changed to
`ec85280a71bf045bf642dbc2c03a96c448dd9476fafe2a6b641371c6c7d8d1b1`.
Owned-process cleanup again reached zero. Raw evidence is retained separately
under `target/supplied-html-krr-0.4.22-calloop-0.14.5/`.

After step seven started, the runner PID 95803 was checked against the receipt's
kernel start time before a one-second native `sample` capture. The 443-sample
browser-worker stack was in KRR `HtmlBrowserSession::resize` and
`HtmlInteractiveSession::traced_layout`. Its text-wrapping path included
`last_fitting_end`, `text_width`, `measured_text_width`, `html_font_runs`,
`resolve_cached_html_face`, and `matching_fallback_face`. The latter reached
`fontdb::Database::with_face_data` / `font_has_char` and repeated file
open/mmap/munmap work. This is an observed rendering hot path, not JS execution
or Chromium startup evidence.

Crucially, the sampled main thread was **not blocked in adapter join**: it was
inside the close helper's resource-check loop, predominantly waiting for the
process-list subprocess (289 samples) or sleeping between checks (151 samples).
The earlier synchronous-Drop hypothesis is therefore not confirmed by this
capture. The precise later blocking stage and the outstanding-resource reason
still require diagnosis; a single one-second sample cannot establish the whole
close timeline. This correction and the raw stack were sent to both existing
owners, with KRR asked to investigate the observed font-fallback I/O path.

On the updated dependency graph, all 42 release runner tests, strict all-target
release Clippy, locked metadata, formatting and all four supply-chain categories
passed. Compatible dependency dry-runs propose zero further updates; the sole
reported newer transitive `generic-array` 0.14.9 is excluded by the published
`crypto-common` 0.1.7 exact `=0.14.7` dependency. Major/pinned/recursive direct
dependency audits and the actual MathJax Bun audit propose no additional update.
None of these checks overrides the failing original document acceptance.

A second trace-only rerun with the same executable/input/request captured the
first three seconds of close at 10 ms sampling intervals (233 samples). It again
failed the unchanged five-second close budget and was externally terminated at
60 elapsed seconds, maximum sampled CPU 81.0%. Raw evidence is retained under
`target/supplied-html-krr-0.4.22-close-trace/`. The main-thread resource loop and
KRR font-fallback hot path were observed again, not a main-thread join wait.

The root harness also needs an identity/precondition audit: its `open_file` uses
the mounted workspace copy, whereas the subsequent `open_url` names the Desktop
original. The close helper closes only the active document but requires *all*
documents and resources to reach zero. Whether this sequence leaves a second,
distinct open document must be established before interpreting the whole close
failure as an upstream product lifecycle defect. This is an open audit, not an
accepted workaround or a relaxed acceptance criterion.

## Corrected close-scope acceptance

Independent source audit confirmed that the mounted copy and canonical Desktop
original are distinct documents. A typed `close_all_documents` harness operation
now closes both under one shared five-second deadline, including resource drain.
The existing single-active-document operation remains available for its original
callers. The acceptance keeps the same original source hash, viewport, fragment,
initial-frame budget and zero-resource assertions. The regression rejecting the
old single-close request failed on the old validator, then passed with the new
contract. The normal host target passed its 24 contract and seven owned-process
tests. All 42 then-current release runner tests and strict all-target release
Clippy passed; the independent diff review found no P0/P1.

The corrected actual request still failed its close budget. The raw log confirms
`closing all documents: count=2`. First frame was 49.369563042 seconds; the
fragment action took 45.106203083 seconds. The external supervisor detected
step-seven timeout with its unchanged five-second budget at 105 total elapsed
seconds, maximum sampled CPU 82.6%. Owned cleanup subsequently reached zero.
Concurrent load varies, so these timings are not a controlled speed comparison.

Evidence is retained under `target/supplied-html-krr-0.4.22-close-all/`. Its runner
SHA256 is `1377c69fec06a5ca153ce9d022fdc99e75d55b2b4bae0bc6445103f985d1ed0b`;
the corrected request SHA256 is
`ec87807c1fb648072c2f269fda1566e200c9e4f2bb8d207980c39918654107e4`.
The worker and updated lock hashes are unchanged from the preceding recheck.
Source HEAD remains `f3d6dadb` with locally modified dependency and harness code,
not a committed release artifact. A close-stack capture was attempted only after
the process had exited; the kernel identity check rejected sampling, so no new
close-stack evidence exists for this corrected request. Do not substitute the
earlier single-close stacks as proof of its exact blocking stage.

KRR Issue #95 is now reopened. The owner received the corrected failure and the
separate font-fallback hot-path evidence; no sibling source was edited here.
Office RSS and canonical fidelity remain separate unmet release requirements.

The final harness source (including a typed-action deadline deserialization
regression) subsequently passed all 43 release runner tests, strict all-target
release Clippy, the normal formatting target and diff whitespace verification.
These local checks validate the harness repair, not the failing real HTML close
or the full dependency-change/release gate. The changes remain on the existing
release worktree with zero stash entries; no master or sibling source was edited.
