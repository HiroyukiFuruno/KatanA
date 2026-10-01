# Self-review: build resource management

## Scope and results

- Existing automatic cargo-sweep calls now use dry-run only. New-profile races
  cannot cause automatic deletion, and Cargo profile locks protect discovered
  profiles during candidate inspection. Unknown directories/filesystems skip
  cleanup; genuine executed sweep errors remain errors.
- Real Cargo build barrier / completed release build / real sweep dry-run and
  ambiguous/symlink target contracts: 3 tests pass. Native `just sweep` safely
  skips the current nested llvm-cov target layout.
- Ubuntu cloud job `110332509535` failed at linking, after a 30 MB free-space
  warning. No failing Rust assertion was observed. The test job now omits only
  dev/test debug information and retains `strip=none`, debug assertions,
  optimization settings, overflow checks and all tests/acceptance/coverage.
- Compact test artifacts have a distinct cache namespace and restore prefix;
  old large debug artifacts are not restored through the earlier prefix.
- Actual workflow contracts: 2 tests pass. Native normal test/check-full
  entries include both contracts; GitHub test job includes its resource contract.
- New Python scripts are below 200 lines, and functions below 30 lines.
  Existing source/API/lockfiles/coverage exclusions/baselines are unchanged.

## Findings and acceptance boundary

Discovered-profile locks alone do not exclude a concurrently created profile.
Automatic sweep is therefore inspection only, not claimed race-free deletion.
Explicit generated-artifact cleanup requires its own scoped, idle ownership
checks. Windows/unknown lock filesystems skip guard cleanup; the existing
Windows test-inclusive verification is not skipped.

The new cloud workflow must still complete on the next formal HEAD. Previous
local full coverage and platform gates do not prove this new cloud result.
No gate was disabled or weakened, and the PR remains Draft.

Windows job `110344558517` subsequently failed before workspace tests because
the new resource contract read the UTF-8 workflow with the Windows cp1252
default. Repository readers in both new contract files now explicitly use
UTF-8. All five focused contracts pass after this repair; a new Windows cloud
run is still required. Missing acceptance artifacts in that failed job follow
from this earlier failure, and are not evidence of an application regression.

## Conclusion

PASS for the scoped resource-management changes and focused contracts;
current-HEAD cloud / final document and distribution acceptance remain open.
