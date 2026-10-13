# Physical footprint budget review

## Finding and scope

Review 5435910718 on commit `3e7722ef2125f2d9367ed3601a783f706a326e93`, thread `PRRT_kwDORm09y86ps_Lb`, identified that the resource-cycle validator recorded physical footprint but enforced growth budgets only for RSS.

The repair adds independent physical-footprint cold and steady checks using the existing recorded numerical budgets: 196608 KiB and 65536 KiB. RSS checks, resource counts, generation closure, artifact identity, and snapshot binding remain unchanged. No application renderer, allocator, worker, or upstream dependency changes are included.

## Verification

The delegated acceptance-gate worker reported these actual execution results:

- Before the validator repair, the new physical-only cold and steady overflow cases each failed with `AcceptanceEvidenceError not raised`; command exit 1. RSS remained within its existing budget and physical footprint exceeded the corresponding limit by one byte.
- After the repair, `test-document-fidelity-acceptance-evidence.py`: 51 tests, 71.449 seconds, exit 0.
- `test-document-fidelity-release-gate.py`: 31 tests, 8.686 seconds, exit 0. The added regression invokes packaged-host mode and rejects physical overflow despite acceptable RSS.
- Targeted Python compilation and `git diff --check`: exit 0.

The boundary and invalid-field cases cover exact limits, a missing field, and bool/float/string values. These are validator test artifacts, not real packaged-application acceptance evidence.

## Main-agent review

PASS: the diff adds two independent constants and two rejection conditions; original RSS budgets are not relaxed. Existing whole-snapshot equality still binds warm and final measurements to the first and last Office close snapshots, including physical footprint. The specification now makes the independent metric checks explicit.

Normal commit/push, individual review reply/resolve, a fresh thread query, Ready required CI, and actual packaged acceptance remain separate release checkpoints.
