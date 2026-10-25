# Original Office and HTML acceptance on frozen 17e9d146

The existing `JOBS=1 CARGO_BUILD_JOBS=1 just test-document-fixtures` target completed with exit0 (root session28186). Source remained clean at17e9d1466d83092f9d28ec060597e95af3ec8bbf throughout build and both tests; no application restart, user settings/Terms manipulation, upstream edits, or acceptance threshold changes occurred. Private original names and paths remain in the local raw log, not this public document.

- Raw: `tmp/issue345-17e9d146-document-fixtures-2026-10-14.log`.
- Locked registry graph remains KDV0.5.12/KRR0.4.23/KUC0.4.1.
- Root Cargo.lock SHA256:31e2f449f83ee4d4efbd5ad1a52209d817db9fd643ae7257cd8805127a5ee7bd.
- Screenshot Cargo.lock SHA256:8fd21cb4b180c862a02203337ac3301de921ef3214ee8b20f2be0b208f3189f4.
- Actual strict optimized arm64 Mach-O worker SHA256:f25a13043281165c730b30d7195fee077753f902186223ef2f0653d054fa940f.

The target's existing HTML host contract prerequisite passed, release worker build completed in30m03s, and external-feature test compile completed in28.74s. The uninstrumented first-frame test accepted every supplied original:

| Format | Items | Surface | First frame ms |
|---|---:|---|---:|
| PPTX | 9 | Page | 6300 |
| PPTX | 11 | Page | 1604 |
| XLSX | 5 | Grid | 27 |
| XLSX | 1 | Grid | 74 |
| PPTX | 26 | Page | 4353 |
| XLSX | 1 | Grid | 7719 |

Exact Office test:1 passed/0 failed/0 ignored,20.12s. Exact supplied requirements HTML test:1 passed/0 failed/0 ignored,10.87s; recorded `#s15` navigation, scroll_y3048.824, content_height9539, sticky_sidebar_ratio0.9352, first_frame_ms10870. Explicit real fixture paths were supplied; these are not missing-input early returns or zero-test filters.

This verifies the existing in-process first-frame and sticky-sidebar contracts only. It does not establish Chrome style/interaction equality, performance non-regression across machines, independent95 scores, all pages/slides, normal-close/resource budgets, full coverage/3OS gates, packaged or clean-machine acceptance, public Release, or whole Issue339/345 completion. DOCX/PDF are not included in these six originals. Previous worker hashes are not relabeled as this strict build.

The additional worker build exposed different default RUSTFLAGS between the direct screenshot entry and Just. Its default alignment is a separate Issue343 concern, not a reason to remove strict warning checks or change the above acceptance result.
