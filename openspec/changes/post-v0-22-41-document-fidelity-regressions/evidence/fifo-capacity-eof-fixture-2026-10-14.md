# FIFO capacity fixture cancellation race

The normal push for source `374787787d1906e8b02ddb262aa5aabef7f2c3ca` terminated with exit 1 before upload. Native UI reported 1232 passed, one failed, two ignored and two filtered tests. The capacity child failed when writing `%PDF-1.7` after both requests had been cancelled: the real reader returned from its blocking open, observed cancellation and closed before the fixture wrote its header.

The fixture now opens and closes the writer without writing unnecessary payload. This releases a blocked open or read through EOF, without treating a broken pipe as success. The capacity refusal assertions, process-wide active-read cleanup, subsequent regular-file descriptor URI check and original five-second deadlines are unchanged. No product API, worker count, queue limit, timeout, ignored test or lint exemption was changed.

## Verification

- Original isolated test: session 23769, exit 0, one passed in 0.06 seconds. The full-gate failure is intermittent; the isolated success does not invalidate it.
- Changed isolated test: session 3900, exit 0, ten consecutive executions passed. Raw log: `tmp/fifo-capacity-eof-repeat-2026-10-14.log`.
- Related intake tests and static checks: session 1178, exit 0; 13 real intake tests, official AST 23 tests, strict UI all-target/external-fixture Clippy, formatting and diff checks passed. Raw log: `tmp/fifo-capacity-eof-target-checks-2026-10-14.log`.
- Failed normal push: `tmp/37478778-normal-push-2026-10-13.log`. Upload did not occur; public branch remains `19e5e1fe7fa61d599cf91432328931b9345e9734`.

## Remaining boundary

Taking the result sender does not prove that the worker has entered the OS open/read call. That pre-existing startup synchronization risk remains tracked; the EOF-only change fixes the observed unnecessary-write race, not every possible schedule. A future synchronization repair must use a genuine state transition and preserve the capacity and recovery assertions, without sleeps, relaxed deadlines or a product test-only port.

Current full gates, current review, packaged acceptance and publication remain incomplete.
