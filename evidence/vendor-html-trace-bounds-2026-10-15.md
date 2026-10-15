# Bounded vendor HTML trace summary

## Accepted review

PR #346 review 5528605915, comment 4279432020 identified a second HTML
diagnostic summary that collected and joined the entire input before taking
80 characters. This is KatanA-owned vendor code, not an upstream wait.

## Repair and scope

The summary now compacts Unicode whitespace incrementally, preserves the
old first-80-character output within the scan budget, and consumes at most
4096 input characters. Budget exhaustion is marked within the same output
limit. The original trace-target predicate is unchanged, including its
full-input `contains` scans; the entire trace is not claimed constant time.

The dependency-free production module's inline tests are compiled directly
with `rustc --test -D warnings` by `just test-commonmark-html-trace`.
The same target is wired to normal local tests and the Ready CI test job.
This neither regenerates the vendor workspace lock nor adds a public API.
Its crate-local function remains inside a private module.

## Verification

- Worker baseline failed with exit 101, then four focused tests passed.
  Those initial standalone Cargo runs regenerated a stale vendor lock and
  are not root-lock verification. The worker restored only its own lock
  changes; the lock is unchanged from HEAD.
- Main's final `just test-commonmark-html-trace`: four tests passed,
  including Unicode, exact 80 characters, bounded huge tail, and the
  whitespace scan budget. No ignored tests or snapshots.
- Root fixed-lock `cargo check --locked -j 2 -p katana-ui --lib` succeeded
  in 11.23 seconds with the normal `RUSTFLAGS=-D warnings` and existing
  debug-profile settings, compiling the actual vendor caller and host.
- An earlier check used a differently spelled RUSTFLAGS value and began
  unnecessary cache rebuilding. Main interrupted that owned process;
  its recovered exit 130 is not a successful check.
- Standalone vendor Clippy encountered pre-existing backend `misc.rs`
  warnings (argument count at 303 and collapsible condition at 426).
  No allow, dependency update, or unrelated backend repair was added.
  The normal pre-commit and push gates remain required.

## Remaining

Normal signed source commit: `d0bb921d`, recovered exit 0. Fresh independent
read-only review found no remaining P0/P1 or required P2 within this concern.
The existing Draft/Ready CI policy contract also passed all five tests.

Normal integration/push, individual review reply/resolve, fresh review
collection, Ready checks, packaged acceptance, and publication are not
proved by these targeted results.
