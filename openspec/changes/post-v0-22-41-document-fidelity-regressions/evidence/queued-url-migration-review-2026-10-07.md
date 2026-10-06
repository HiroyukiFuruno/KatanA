# Queued URL navigation during format migration

## Current review and ownership

PR346 review5432594216 on public5a21245c reported P2 thread
PRRT_kwDORm09y86pmEZA / discussion4198748139. When the first queued response
retagged an HTML tab as PDF or Office, later requests still targeted its old
path and the closed-tab guard discarded them. This is a KatanA-owned routing
repair, not an upstream renderer change.

## Repair

- Retarget pending requests only after an actual clean-tab migration succeeds.
- Keep dirty originals, existing destination collisions and unrelated requests
  unchanged. Closing the migrated tab cancels its retargeted requests.
- A later HTML response migrates a binary tab back to an HTML identity, keeping
  its pin and the existing path-reference migration behavior.
- No public API, timeout, ignore, quality threshold or dependency change.

## Reproduction and verification

- Original product: session34217 exited101, 24 passed and three assertions failed.
  `tmp/url-queue-migration-review-red-corrected.log` retains the actual failures:
  newer response lost, pending request lost and PDF identity retained for HTML.
- Earlier session72458 was a compile failure from an unused test variable, not
  product assertion evidence.
- First candidate92652 exited101, 26 passed and one failed. The DOCX fixture
  incorrectly used a PDF URL; the existing remote classifier rejects that
  mismatch. Fixture URLs now use their actual format extensions. This failure
  is retained in `tmp/url-queue-migration-review-green.log`.
- Corrected candidate46654 exited0: all27 URL routing tests passed in2.75seconds.
  `tmp/url-queue-migration-review-green-corrected.log`.
- Four new private regressions cover PDF/PDF, DOCX/DOCX, PDF/DOCX and DOCX/PDF
  queues, binary-to-HTML identity/content/pin, close cancellation, and dirty or
  colliding destination preservation. Both private test files remain below the
  existing300-line limit.
- Minimal PDF/ZIP payloads exercise classification and routing only; they are
  not Office rendering, legal-document or packaged-host acceptance evidence.

## Remaining delivery

Final static session22420 exited0: AST23, impacted strict Clippy, format and
format-check passed. Manual diff/call-site review confirms the migration guard
and close cancellation remain intact; the new tests invoke existing actions
without public test ports, mocks or relaxed assertions. Source comments follow
the latest explicit Japanese-comment instruction. Master is clean and stash0.

Source and private regressions were integrated by ordinary signed commit
`b0088fb8f38f03ce2e415667cece05b8efaf8c52` (session49488 exit0, signatureG).

Ordinary push, individual reply/resolve and
fresh review collection, current-source CI/coverage/package and actual packaged
host acceptance remain separate checkpoints. The passing public5a coverage
and earlier packages must not be reused for this changed graph. The human app
and Terms have not been operated or accepted by this task.
