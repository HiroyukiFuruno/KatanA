# Supplied Office first-frame results

## Method

The supplied HTML/XLSX/PPTX files were opened through the KatanA host with
`DEBUG=true`. Each Office document passed through KatanA source intake and a
real KDV 0.5.5 `DocumentSession` using the built `kdv-office-worker`. The
measurements below are the current frame-receipt traces; KatanA screenshots were
captured for every file. A second single-process run opened and closed all seven
files in sequence, sampled RSS after every close, and required the KatanA-owned
preview, HTML/document surface, worker, frame, texture, and cache counters to
return to zero before continuing.

## Results

| File | Format | Items | Surface | First frame |
| --- | ---: | ---: | --- | ---: |
| `要件定義書_v1.0.html` | HTML | — | HTML frame | 1,070 ms |
| `shopchannel_analysis.xlsx` | XLSX | 5 | Grid | 71 ms |
| `librechat_entra_oidc_vs_saml.pptx` | PPTX | 11 | Page | 5,239 ms |
| `【681303_発注依頼書】...xlsx` | XLSX | 1 | Grid | 67 ms |
| `【チャット型AIエージェント】...pptx` | PPTX | 26 | Page | 4,337 ms |
| `libre-chat_vs_loom.pptx` | PPTX | 9 | Page | 2,139 ms |
| `視聴購入data_セグ別201805_20260803.xlsx` | XLSX | 1 | Grid | 713 ms |

All seven supplied HTML/XLSX/PPTX files produced a frame. No ZIP archive error
reproduced in any supplied XLSX/PPTX. The folder contains no DOCX fixture, so a
separate deterministic data-descriptor DOCX was added and reproduced the ZIP
failure; see `docx-data-descriptor.md`. PPTX latency is isolated to the
KDV/Office conversion path; current XLSX frame receipt is sub-second for all
three supplied workbooks.

## Resource profile

| Boundary after close | RSS | Delta from cold idle | Retained KatanA resources |
| --- | ---: | ---: | ---: |
| Cold idle | 105,808 KiB | — | 0 |
| Supplied HTML | 233,088 KiB | +127,280 KiB | 0 |
| Small XLSX | 277,184 KiB | +171,376 KiB | 0 |
| 5.8 MiB PPTX | 283,824 KiB | +178,016 KiB | 0 |
| 670 KiB XLSX | 310,896 KiB | +205,088 KiB | 0 |
| 18 MiB PPTX | 303,296 KiB | +197,488 KiB | 0 |
| 40 MiB PPTX | 544,032 KiB | +438,224 KiB | 0 |
| 82 MiB XLSX | 374,960 KiB | +269,152 KiB | 0 |

The final cold-to-corpus delta exceeded the unchanged 256 MiB budget by 7,008
KiB. This does not present as a KatanA surface/session reference leak: every
owned counter and Office worker was zero at each boundary, and RSS fell by
169,072 KiB when the final workbook created memory pressure. The dominant cold
HTML step is owned by KRR 0.4.19: its process-global HTML font database loads all
system fonts into a `OnceLock`. The measured +127 MiB first-HTML step and the
large number of `Helvetica Neue` to `Arial Unicode MS` fallbacks are tracked in
[KRR issue #74](https://github.com/HiroyukiFuruno/katana-render-runtime/issues/74).
An attempted macOS allocator pressure-relief call returned zero bytes at every
close and was rejected rather than retained as an ineffective workaround.

The committed mixed-document lifecycle scenario now has two independent
budgets: at most 192 MiB for cold HTML/Office initialization and at most 64 MiB
growth across ten additional warm cycles. The seven rendered screenshots are
kept in the ignored local evidence directory
`output/playwright/supplied-document-acceptance/`; all are non-blank and show
the expected first HTML page, XLSX grid, or PPTX first slide. Objective
Office-vs-reference geometry and missing-element scores remain unverified until
the owner-layer reference renderer and KDV 0.5.6 result are available, so task
4.4 remains open.
