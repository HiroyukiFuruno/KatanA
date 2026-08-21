# Local Verification

- The registry KDV `0.5.4` checksum is `f7bfed62785f4274102a86d1cfaf7ec66850693156dd8af2659a6fea7d3b18ca` in both KatanA lockfiles.
- The transitive official `office2pdf 0.6.7` checksum is `0cd39889efe9f4bc36ea89ffc30ad5ecf7e1cd3a33b5af76604d54ca26f764c3` in both lockfiles. No `office2pdf-katana` package, path source, or git source resolves.
- `python3 scripts/screenshot/generate_external_hyperlink_pptx.py --self-test` passed. The generated PPTX contains one standard transitional hyperlink relationship with `TargetMode="External"`; the source fixture SHA-256 remains `bfd22adbd4909c3906bdd2bdbcd7dac613c819044afc3e4985752dcfd268b0b8`.
- The macOS headless scenario completed all 37 steps. The external-hyperlink PPTX produced its initial `Page` frame in `0.862s`, navigated to slide two, and changed `2004683` pixels. Visual review confirmed a rendered slide rather than a KDV diagnostic surface.
- `just update` completed with the exact KDV requirement retained at `=0.5.4`.
- `just coverage` passed with strict document surface coverage at `100%` and `0` uncovered lines. The prior run was retried after its instrumentation target exhausted the disk; no test, coverage, or exclusion rule was changed.

Cross-platform CI and public release evidence remain pending and are intentionally not claimed here.
