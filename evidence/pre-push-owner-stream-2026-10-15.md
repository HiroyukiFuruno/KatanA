# Pre-push owner stream preservation

## Scope

Issue #335 requires the retained native hook to run before added admission
checks. The dispatcher previously rejected oversized input and disabled
Lefthook before invoking that owner, resolved a relative executable through
PATH, and converted signal termination into an ordinary Python exit status.

## Repair

- Reuse the reference dispatcher's complete-input streaming primitive.
- Capture at most 1 MiB plus one byte for additional pre-push inspection.
- Run the absolute, validated owner with its original arguments and the
  verification-session environment; strip only cleanup intent keys.
- Preserve nonzero owner exit and signal termination before added policy.
- After owner success, still reject oversized input, disabled Lefthook,
  changed HEAD, and invalid verification receipts. Do not invoke the checker
  for the first three rejection cases.

## Evidence

- Main's corrected native regression: five tests, four assertion failures
  and one missing-owner-log error before the repair. The relative-path case
  actually passes a basename and observes the PATH shadow's exit 17.
- Focused pre-push discovery: 21 tests passed in 7.623 seconds.
- Final release Python discovery: 307 tests passed in 128.079 seconds;
  raw output: `tmp/pre-push-owner-stream-final-tests-2026-10-15.log`.
- Changed Python sources and tests parse successfully, remain below 200
  lines per file and 30 lines per function. No Mock, skipped test,
  verification receipt exemption, or hook bypass was added.

## Remaining

Normal signed source commit: `d17803c7`, recovered exit 0. Fresh independent
read-only review found no remaining P0/P1 or required P2 within this concern.

These are dispatcher component results, not completion of Issue #335.
Production activation, publication admission, operation serialization,
local/remote deletion and retention, scoped pruning, workflow integration,
current public review, and release acceptance remain separate requirements.
