# Failed local image label geometry

## Finding and scope

Current review 5435243464 on ebdce22a reported comment 4201042151, thread PRRT_kwDORm09y86prljY. Both failed-image rendering paths displayed the existing error label but returned no rectangle. The actual section caller consequently omitted drawn-state completion and the document anchor.

Source commit: 9f646b2c8f84497f23c67625dc989e957368dd79 (ordinary signed commit, signature G; commit session 62039 exited 0).

The repair returns the existing label rectangle in the two failed branches only. Pending requests still return no rectangle; successful textures, asynchronous decoding, retry policy and upstream dependencies remain unchanged.

## Verification

- Genuine pre-repair runtime RED: session 7946 exited 101, one test passed and three assertion tests failed. Raw log: tmp/failed-image-rect-runtime-red.log.
- Initial fixture compilation failure in session 98973 is retained separately and is not counted as runtime RED.
- Post-repair session 51072 exited 0: all four caller regressions passed.
- Final session 83989 exited 0: four caller regressions, 38 local-image-loader regressions, 31 AST/token checks, strict impacted lint, formatting and diff checks passed. Raw log: tmp/failed-image-rect-final.log.
- Assertions preserve positive error-label geometry, section drawn state, the 0..1 anchor range, no failed texture and the pending-request no-rectangle contract. Tests use actual missing-file decoding and the existing egui/section caller, not a mocked renderer.
- Main-agent self-review: tmp/failed-image-rect-self-review.md.

## Evidence boundaries

This is focused host-side regression evidence, not packaged, human-device, all-OS or independent fidelity-score acceptance. Coverage collected for ebdce22a is not evidence for this later source change. Publication, fresh review-thread retrieval and current-HEAD release gates remain separate steps.
