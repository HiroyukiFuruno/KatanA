# Public KRR 0.4.23 downstream acceptance

## Publication and dependency identity

- KRR PR105 was merged at 2026-10-05T08:42:35Z after all three OS CI jobs, preflight and current review. Live reviewThreads had no unresolved thread and no next page.
- Release run37285317328 succeeded after the asset-crate token-permission failure was repaired by the existing owner/user. GitHub Release v0.4.23 is non-draft, published at 09:28:48Z. The public sparse index marks the runtime and assets non-yanked.
- Root uses exact registry `katana-render-runtime = "=0.4.23"`. Both application and screenshot locks contain runtime checksum `8e5e0803f343d781dc015d1fe21b052492d67579ba303cf86203f60455852e00` and assets checksum `729af5d660bb6f439ced0a7c56d86ad1cd15b6f1092becc70afd97d41c5a6073`. New transitive nix is0.31.3. Locked metadata retains public KDV0.5.11, KUC0.4.1, office2pdf0.8.1 and singleton V8 152.2.0. Existing internal/vendor patches are unchanged; no sibling path/git override is introduced.

## First original-HTML execution

The unchanged `scripts/ci/html-fixture-host-acceptance.sh` accepts the original supplied HTML, original1280x900 viewport and `#s15`, retaining first-frame/navigation60s and close5s limits. Release worker build4m49s and separate screenshot-root build5m00s succeeded. Runtime DEBUG was not enabled.

- Input, viewport, fragment, every operation deadline and process-group closure validation passed. Runner exit0, all9 operations completed, original HTML first frame1.427s, fragment navigation1.841748083s, normal CloseAllDocuments0.922562375s.
- Final snapshot: all preview/HTML/document surfaces, worker/session counts, retained artifacts, cached pages/cells, host frames/textures/cache entries0. External supervisor observed exit and zero live process-group members, not timeout cleanup.
- Entire runner observation6s, sampled maximum CPU92.9%. Closed-idle RSS315504KiB alone is not a memory-increase verdict. These observations do not establish independently scored fidelity, continuous native event-loop responsiveness or every possible escaped descendant.
- Artifact base source HEAD ad2780ee with the explicitly uncommitted public dependency update. Runner SHA256 `040196a86749112988a2097ac0c8fe2efa0933025165b143456c19c1cbd82c06`, worker `c249110cd926465bc7e7df8e8eb290dd102b3e43667bcb404eaebe132ad51671`, screenshot lock `7647a2a23824b884f8e63960f2ee418dafa861e78933fc9d327e03342f5cbac8`. Raw evidence: `tmp/public-krr0423-html-2026-10-05/` and its sibling build log.

## Full dependency re-audit and remaining gates

Application and screenshot compatible update dry runs proposed powerfmt0.2.0→0.2.1. It is now updated in both locks, and a fresh original-HTML rebuild/re-run is in progress; the first execution above predates that leaf update and is not substituted for final-graph evidence. Major/pinned direct upgrade dry runs proposed no update (65 application /10 screenshot direct packages current). The actual JS package `vendor/mathjax_svg/js` has no outdated package and TypeScript checking passes. Root Bun's ancestor-package Biome result is not a KatanA package audit; KatanA tooling already pins Biome2.5.15.

Supply-chain advisories/bans/licenses/sources pass on the initial public KRR graph. HTML input/deadline24 and real process7 contracts, repository formatting and JSON51 checks pass. Changed-final-graph full gates, Office inputs/RSS, independent95-point categories, canonical adoption and all packaged CPU/OS/clean-machine/distribution checks remain required. Source rendering screenshots and this in-process CLI are not packaged-main acceptance. No threshold, consent or manual-target publication-policy choice has been changed.

## Final-lock original-HTML execution

Both compatible update dry runs now propose zero package updates. After powerfmt0.2.1, the same unmodified driver and original request complete again with exit0: first frame1.352s, fragment navigation1.703966916s, normal close0.959171167s, all9 steps within deadlines, zero residual process-group members and idle resource counts. Observation6s and sampled maximum CPU88.7% are recorded, not used as continuous packaged-UI proof.

Final runner SHA256 is `0af3fde0759efdf5ff4bc0ed12d6e87398d53a05487ad678cca34ea579108910`; worker is `830426ad5c43e705ec69f7cc67d75952e0578b175357c1403aa5a04133d52955`; screenshot lock is `67dcd994d2b0a3d46ab5c69e9975a508d9229d7e6c92e1e4d09b4e23f4bf44c2`. Raw records are in `tmp/public-krr0423-powerfmt-html-2026-10-05/`. Original Office5 repeated twice, multi-format inputs, legal data-descriptor DOCX, actual sheet switching and three cold/warm cycles are now running sequentially using these exact final binaries. Their completion is not yet claimed.

## Final-binary Office and input regressions

Normal-runtime Office5 completes twice with exit0: RSS105328→291456KiB (delta186128) and105136→297440KiB (delta192304), both below the unchanged196608 limit. Second-run first frames are0.079/0.105/4.001/2.309/1.595s. This does not establish a particular allocator fix or subsecond Office performance. Prior failures and near-limit variation remain relevant.

Legal data-descriptor DOCX and actual Sheet0→1→0 complete with exit0. PPTX/XLSX/original-large-PPTX each complete cold1+warm10 mixed HTML/document cycles with exit0 and11 actual frames of each kind. Cold/warm RSS deltas are respectively74848/4912,127424/34016,155280/-19696KiB, retaining196608/65536 limits. End counters are0 and normal-close5s is unchanged.

The initial multi-format invocation fails before product execution because `target/screenshot-fixtures/representative-with-external-hyperlink.pptx` was absent. This is not counted as a successful suite or a renderer defect. The existing `scripts/screenshot/run.sh` generates this fixture and a retry is in progress. Raw initial failure is preserved; the original fixture,38-operation request and expected external-link behavior are not replaced or weakened.

The official-entrypoint retry completes all38 operations with exit0, including real bottom-tab input, PDF/DOCX/PPTX page navigation, external hyperlink preflight and direct-URL recovery. Its distinct raw log is `tmp/public-krr0423-multi-format-retry-2026-10-05.log`. The final graph is now frozen for the official `CARGO_INCREMENTAL=0 JOBS=2 just check-full` sequence; log `tmp/public-krr0423-check-full-2026-10-05.log`. Whole coverage/platform/supply-chain completion is not yet claimed.

## Current full-gate progress

The official full sequence passes resource and cargo-sweep contracts, formatting, strict workspace Clippy and all8 fixture integration tests. Native full coverage passes core215, linter56, platform117, UI1046 (the existing2 ignored tests unchanged), main17, real export13, UI parallel143 (existing2 ignored) and serial18. Meaningful uncovered lines are0; strict document surface coverage is100% with0 uncovered lines. Raw final-graph coverage JSON is15444408bytes, SHA256 `6feec8b6eed7d0351bf1921a7182b4255dd4a7bafe4e6e770dd05d6a95c344a1`, saved at `tmp/public-krr0423-full-coverage-2026-10-05.json`.

The full sequence finishes with exit0. Linux actual-worker workspace tests pass, including UI1030 (existing2 ignored), actual export and integration suites. Windows test-inclusive MSVC cross-compilation passes; this is not execution on native Windows. Final-graph supply-chain advisories, bans, licenses and sources all pass. The log ends with `All checks passed`. Native coverage and these platform checks do not substitute for packaged acceptance, independent95-point fidelity or the release DoD.

## Self-review

### No issues

- The diff changes only the exact public runtime version, the two official locks and acceptance documentation. Application and screenshot consumers use the same published runtime/assets checksums; there is no new sibling path/git override or public API change.
- No product/test source, timeout, RSS limit, coverage exclusion or quality score is weakened. Version0.22.42 remains consistent with the release branch and Info.plist.
- Real-input regressions, dependency audit, native full coverage, strict lint, Linux execution, Windows cross-compilation and final supply-chain checks pass. Initial invocation failure is retained separately from the successful official retry.

### Findings and remaining scope

- Office first frames can still take roughly4s; these passes do not prove a particular RSS cause has been repaired. Independent score categories, updated packaged binaries on every declared target, clean-machine acceptance and publication remain open.
- KDV next-version work stays with its existing owner. The manual partial-publication P2 remains a user-policy choice, not silently resolved by this dependency update.

### Conclusion

PASS for this dependency-adoption and evidence diff; NOT complete for the overall release.
