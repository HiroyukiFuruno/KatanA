# Self-Review: prepared Office grid border projection

## Verified

- The worker prepares typed borders once for each frame and moves frame/cache together through the generation-bound event. The UI consumes the prepared cache without repeated viewer lookup.
- Stale events preserve both the current frame and cache; accepted events replace both. Actual native focused tests pass62 cases after the final private-constant visibility correction. All fourteen styles assert their actual line width and first segment length; zero-length and offscreen output and coordinate mismatches are covered.
- Actual painter output checks four sides, color, width, double separation and clipped cell geometry. Unknown style/color and cache mismatches remain explicit typed failures; no silent cache fallback is added.
- The public viewer surface retains merged-cell geometry. Real XLSX worker/filter tests check prepared-cell count against the received grid.
- Responsibilities are split into style, preparation, paint, worker projection and worker session modules. AST23 passes, normal native tests pass, workspace strict Clippy and full format checks pass. No threshold, ignore, exclusion or production visibility is relaxed to accommodate tests.
- The new Office diagnostic validates exact event fields, percent-encoded URI, per-poll subtree totals and verified real-child cleanup. Parser and ownership regressions pass. Native Terms-modal progress is explicitly not Office acceptance.
- Biome2.5.15 schema migration preserves the existing rule levels and limits. Normal JSON/JS tooling checks pass.

## Remaining release conditions

- KDV0.5.7 only exposes per-cell linear border lookup. The current one-time worker preparation is provisional and must use the published batch API from KDV Issue56 before final performance acceptance.
- Named Office fonts, objective fidelity, post-change coverage/platform checks and packaged real-input acceptance remain open in tasks.md.
- KRR's required next public version is not yet available. No private path/git override or sibling implementation is introduced.
- The native Office diagnostic requires human Terms consent. It does not change the consent setting or automate approval.

## Registered Office family follow-up

- Actual egui layout fails before the fix because a registered named family becomes Proportional. The repaired lookup borrows `FontsView::definitions().families` and selects an existing Name case-insensitively. No family-vector clone, font file read or new font payload is introduced.
- Two new tests cover registered identity/resource counts and unchanged unregistered-family fallback. Document-surface focused64 tests and strict Clippy pass; AST23 passes after replacing test-only hardcoded colors with the existing theme color. Post-change complete coverage/platform runs are in progress.
- This does not claim that absent Aptos/Calibri fonts, name-table family registration or font weight selection are fixed. Full source-renderer fidelity remains open.

## Conclusion

The local changes are suitable for formal incremental commits after targeted verification; the release is not ready. Full gates, public dependency adoption, current PR review and distribution evidence are still required.
