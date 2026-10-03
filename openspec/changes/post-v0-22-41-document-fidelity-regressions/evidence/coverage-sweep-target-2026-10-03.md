# Discover and lock the known coverage Cargo target

After separating the Python environment, actual `just sweep` still safely
skipped because `target/llvm-cov-target` contained the ordinary coverage
`debug` and Cargo `tmp` directories. The scanner interpreted its hyphenated name
as a target triple, which only permits debug/release children.

The scanner now treats this exact known coverage root as another Cargo target
and recursively applies its existing profile discovery and validation. It does
not exclude coverage or search arbitrary debug payload trees. Discovered
coverage profiles are included in the same complete profile-lock collection.
Unknown directories, root/profile/fingerprint/lock symlinks and invalid files
retain their existing fail-closed behavior. Actual sweep remains dry-run only.

## Verification and self-review

- New `test_coverage_target_profiles_are_discovered_and_locked` is RED on the
  old scanner with the same ambiguous-profile error. It then verifies native
  plus coverage/native and coverage/triple profile discovery, a real held
  coverage lock preventing sweep, and acquisition/release of all three locks.
- New negative regression rejects unknown coverage directories, profile files,
  invalid fingerprint files and symlinked coverage roots.
- Final `just test-cargo-sweep-contract` passes all six tests, including the
  existing real Cargo build and triple locking contract. Raw RED and GREEN:
  `tmp/coverage-target-sweep-red.log`,
  `tmp/coverage-target-sweep-final-green.log`.
- Actual `just sweep` exits zero with `Running cargo-sweep with all discovered
  profile locks held` and `Would clean: nothing`, not a safe skip:
  `tmp/coverage-target-actual-sweep-green.log`. Nothing was deleted.
- Full acceptance/release-gate tests are rechecked against the final scanner in
  `tmp/coverage-target-evidence-final-green.log`.
- Self-review PASS for the bounded discovery fix: same validation, complete
  lock collection and dry-run policy; no cleanup exclusion, threshold reduction,
  source/fixture mutation or new worktree/stash. Actual nested layout was read
  independently and checked by the main agent; an unobserved arbitrary nested
  debug-target generalization was not adopted.

This fixes local cache-maintenance discovery, not HTML performance, original
Office fidelity, packaged acceptance or the pending public release.
