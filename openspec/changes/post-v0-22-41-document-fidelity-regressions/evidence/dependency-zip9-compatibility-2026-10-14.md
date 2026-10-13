# ZIP9 dependency compatibility

Issue: https://github.com/HiroyukiFuruno/KatanA/issues/339

The existing worktree received compatible dependency updates: direct ZIP9.0 and UUID1.28, with both root and screenshot locks refreshed. Those input changes were preserved. Published KDV0.5.12, KRR0.4.23 and KUC0.4.1 were not replaced with unpublished sibling dependencies.

The changed ZIP entry-name API returns a Result. The previous dereference failed compilation at the update extractor (session66595, E0614). The extractor now propagates the error through its existing Result path, retains directory filtering and enclosed-path checks, and still refuses traversal entries.

The real ZIP regression creates a nested file and a traversal entry, extracts the archive, compares exact nested bytes and verifies that neither outside path was written. It passed with the two existing update-download tests: session26053, three tests passed. Official session94106 passed AST23, impacted strict Clippy, formatting and diff checks on the updated graph. Signed normal-hook commit `e3da3d4d` integrated this concern (session63836, exit0).

This targeted migration does not prove all workspace tests, screenshot regeneration, packaged acceptance or required Ready CI. The pre-update capture and gates are not reused as evidence for this dependency graph.
