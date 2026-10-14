# Issue 350: standalone paint-metrics graph

Current HEAD 42886a2e review 5524700208 identified comment 4276299202:
the screenshot dependency graph does not substitute for paint-metrics' own
independent lock. This is part of the current full dependency gate scope.
The release-flow contract first failed on the missing third command, then
all ten tests passed after the third locked cargo-deny invocation was added.

The official shared gate session 52448 exited 0. Root, screenshot and
paint-metrics each reported advisories/bans/licenses/sources OK in
`tmp/paint-metrics350-three-graph-supply-chain-2026-10-15.log`.
Policy, advisory exceptions and Draft CI conditions are unchanged.

Cargo update session 27498 exited 0 and selected eight compatible updates
for the standalone paint lock: either 1.19.0, serde_json 1.0.154, smallvec
1.16.3, syn 3.0.9, unicode-ident 1.0.27, uuid 1.28.0, zerocopy and
zerocopy-derive 0.8.63. The egui 0.36.2 pin is unchanged.
Official `just test-paint-metrics` session 54982 exited 0: two tests,
0.03 seconds. This is not a visual acceptance or all-OS performance score.

Paint lock SHA256:
`017c8018a8e2bbc13b2a015630f2b365972fbd12faea92eb8c27d9fc48c02ecc`.
Root/screenshot locks remain at their recorded hashes 56bac083/ef715177.
Tracked vendor locks also exist; this three-graph result does not claim to
audit hypothetical standalone vendor invocations. The current KatanA Just
and CI entrypoint search found no independent vendor-manifest command.

Normal integration/upload, individual thread reply/resolve and fresh current
review remain separate. Ready CI, packaging, public assets and postprocessing
are still incomplete.

Normal signed commit `cb5a6282250289203d029f334194524d130f897c` integrated
the third gate, contract and compatible lock updates (session 90851 exit 0,
signature status G). Independent final read-only review found no scoped
P0/P1. Upload, individual reply/resolve and public release are not yet proven.
