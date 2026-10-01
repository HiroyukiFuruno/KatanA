# Mixed document resource-cycle evidence

## Harness contract

- Request: `scripts/screenshot/examples/document-resource-cycle.json`.
- A lightweight Markdown tab is closed before the cold baseline so no document
  preview is active. One HTML/XLSX warm-up then establishes the steady idle
  baseline.
- Ten measured cycles open an HTML frame, close it to zero retained resources,
  open an XLSX frame, and close it to zero retained resources.
- Close completion is polled from UI frames until the active document,
  preview, HTML/document surface, worker process, frame, texture, and cache
  counts are all zero. A fixed sleep is not accepted as close completion.
- The final assertion requires ten observed HTML frames, ten observed Office
  frames, continuing UI frame progress, and a bounded steady RSS delta.
- A separate cold assertion bounds first HTML/Office initialization to 192 MiB;
  it does not hide one-time renderer initialization inside the warm baseline.

## Local result (2026-08-29)

- Cold idle: `rss_kib=107920`, `ui_frame=405`, all retained resource counts `0`.
- Warm idle: `rss_kib=171040`, `ui_frame=414`, all retained resource counts `0`.
- Cold HTML/XLSX initialization: `63,120 KiB`, below the `196,608 KiB` budget.
- Final idle: `rss_kib=171552`, `ui_frame=484`, all retained resource counts `0`.
- Steady RSS increase: `512 KiB`, below the `65,536 KiB` regression budget.
- UI heartbeat progress: `70` frames, above the required `20` frames.
- Observed generation progress: HTML `10`, XLSX `10` after the baseline.
- Office generations 2 through 11 each emitted `document_surface_drop` and
  `document_session_closed`; no `kdv-office-worker` remained alive.
- Result: pass.

## Dependency-audit rerun (2026-08-29)

- Cold idle: `rss_kib=108192`, `ui_frame=405`, all retained resource counts `0`.
- Warm idle: `rss_kib=170720`, `ui_frame=469`, all retained resource counts `0`.
- Final idle after ten measured HTML/XLSX cycles: `rss_kib=170960`,
  `ui_frame=539`, all retained resource counts `0`.
- Steady RSS increase: `240 KiB`, below the unchanged `65,536 KiB` budget.
- UI heartbeat progress: `70` frames; HTML and XLSX generation counts both
  advanced from 1 to 11.
- Every XLSX generation emitted both `document_surface_drop` and
  `document_session_closed`; no office worker, surface, frame, texture, or
  cache entry remained retained.
- Result: pass.

## Current release rebuild (2026-08-29)

- Both KatanA and `kdv-office-worker` were rebuilt in release mode from the
  post-self-review source before this run.
- Cold idle: `rss_kib=107728`, `ui_frame=405`, all retained resource counts `0`.
- Warm idle: `rss_kib=170768`, `ui_frame=604`, all retained resource counts `0`.
- Final idle after ten measured HTML/XLSX cycles: `rss_kib=171680`,
  `ui_frame=674`, all retained resource counts `0`.
- Steady RSS increase: `912 KiB`, below the unchanged `65,536 KiB` budget.
- The first cold XLSX frame took `3.857 s`; warm XLSX frames then took
  `42-45 ms`, isolating the avoidable latency investigation to initialization
  rather than steady host projection.
- UI heartbeat progress: `70` frames; HTML and XLSX generation counts both
  advanced from 1 to 11. No office worker, surface, frame, texture, or cache
  entry remained retained.
- Result: pass.
