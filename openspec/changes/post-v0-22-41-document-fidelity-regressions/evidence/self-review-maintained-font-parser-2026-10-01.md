# Self-review: maintained font metadata parser

## Scope and cause

Normal push of `f9372efc` failed with exit 1 in Draft preflight: the newly direct `ttf-parser 0.25.1` dependency triggered RUSTSEC-2026-0192. Its removal from direct dependencies is required; no advisory ignore, source-policy exemption, or gate reduction is added.

Production metadata parsing and the real Ubuntu OS/2 regression now use `skrifa`. The official sparse index reports latest non-yanked `0.47.0`, MSRV 1.85, compatible with the workspace MSRV 1.95. The direct dependency enables only `std`; font auto-hinting is not used by this metadata-only path. Root and screenshot locks add `skrifa 0.47.0` and `read-fonts 0.44.0` and remove only the UI's direct `ttf-parser` edge. Existing transitive parser consumers remain upstream-owned; their presence is not misreported as removal from the entire graph.

## Static and supply-chain verification

- Official `skrifa 0.47.0` / `read-fonts 0.44.0` source confirms collection-index handling, typographic/family identifiers, localized strings, English preference, weight/style attributes, fixed-pitch metadata, and table-directory offsets used by the migration.
- Generic chains, payload Arc ownership, the one-time initialization call, and DEBUG-only diagnostics remain unchanged. No per-frame I/O, extra `set_fonts`, OS font payload loading, or glyph substitution is introduced.
- Both Cargo roots resolve with `--locked`. Latest dependency graph passes advisories, bans, licenses, and sources with the existing supply-chain configuration.
- `cargo fmt --all --check` and `git diff --check` pass.
- Latest `skrifa 0.47.0` real font-loader suite: 24 passed, zero failed, 18.82-second build and 0.03-second execution. Test-inclusive strict Clippy for UI/core passes (11.66 seconds); AST suite passes all 23 contracts (10.41 seconds).
- Real-font regressions retain regular metadata aliases, actual egui layout, payload/chain preservation, invalid/truncated/index rejection, and OS/2 Bold/Italic rejection. OS/2 classification fixtures are not real Bold glyph-fidelity evidence.

## Pending verification and release boundaries

Changed-source full coverage and the next normal push are not yet verified for this migration. KRR's normal push completed successfully and its owner restored 4.4 GiB of disk headroom before the focused Rust verification above; KatanA did not start a conflicting Rust build. The last failed push leaves remote `e602456a` and two unresolved review threads; local fixes alone are not reply/resolve or release completion.

The separate Office real-face implementation, published KDV/KRR adoption, packaged document acceptance, fidelity score, clean-machine matrix, final review/CI, public assets/checksums, and cleanup remain open in `tasks.md`.

## Conclusion

PASS for the targeted migration checks, with no remaining finding in this diff review. Full coverage, normal push, public-dependency adoption and release remain incomplete.

## Independent review follow-up

Independent review found one P2: `skrifa` classification can return Normal when OS/2 does not advertise italic/oblique, while the post table contains a nonzero italic angle. The previous parser rejected this combination, so style alone was not a compatible exclusion predicate.

The real Ubuntu fixture now sets OS/2 Normal and post angle -12 degrees. Its metadata is asserted through the real parser. Before the fix, the regression exits 101 because a Regular Ubuntu alias is wrongly registered; after checking post angle as well, all 25 font-loader tests pass. Strict test-inclusive UI/core Clippy passes (6.18 seconds), and all 23 AST contracts pass with no threshold/exclusion changes.

Independent re-review confirms the original P2 is repaired and finds no new P0/P1/P2 in the two-file follow-up. Its review is source/diff evidence; the executed tests above were run separately by the main task.

The normal push of `3b21a4db` was interrupted with exit 130 to repair this finding before publishing more changes. The exact owned test container `7b7689e1a448` was verified by compose working-directory/command identity and stopped; `--rm` removed it. No cargo/rustc/lefthook process or test container remained. This interrupted gate is not a successful full verification; remote is still `e602456a`.

Valid nonzero TTC collection index and multilingual family selection remain explicit verification gaps, not proven by bad-index/default-font tests. True Bold glyph fidelity is still separate from these classification tests.
