# Canonical capture source integrity guard

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/338

The official candidate generator now records a clean Git HEAD/status before its regular runner and verifies the same HEAD and clean status immediately after capture, before normalization. A tracked worktree change, staged change, nonignored untracked source or new commit rejects the candidate. Raw diagnostic output is retained. Repository-local output must be actually ignored under tmp/ or target/; no arbitrary source exclusion or branch exemption is added.

The identity file uses a strict schema version integer and full lowercase Git object ID. Invalid JSON, extra fields, bool/float versions and malformed IDs are refused. The identity is an index for capture provenance, not permission to skip build, review, CI or acceptance checks.

Main independently ran the nine real temporary-Git tests successfully, including three distinct changes introduced after begin, HEAD changes, stable ignored/external output and invalid identities. Existing release-flow nine tests, Python compilation, generator Bash syntax, preflight Zsh syntax and diff checks passed. The tests are registered once in the existing multi-format preflight stage.

Evidence: `tmp/issue338-source-identity-guard-main-2026-10-14.log` in the existing release worktree. This guard checks before/after state; it does not detect every transient change that is fully restored between those checks. A coordinated source freeze is still required during the real build/capture. It does not prove packaged execution, independent visual95, reference adoption or final release acceptance.

Normal-hook signed commit `153f32ed` integrated the guard and its one preflight registration (session18762, exit0). Real capture on the new committed graph remains to be recovered separately.
