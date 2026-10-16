# macOS package: frozen cbc19648 structural verification

Source: `cbc19648855b376bc87d648b3dd2adb7a05e459f`, clean throughout the build;
both tracked lockfiles remained unchanged. Existing command:
`CARGO_BUILD_JOBS=1 JOBS=1 just VERSION=0.22.42 package-mac`.
Session 8383 actually exited 0. Native bundle build finished in 1m48s,
ARM target build in 6m54s, and the cold Intel target build in 29m15s.

Bundle: `target/release/bundle/osx/KatanA Desktop.app`.
Independent non-launch verification actually exited 0:

- `lipo -archs` reported `x86_64 arm64` for both KatanA and kdv-office-worker.
- `codesign --verify --deep --strict --verbose=2` validated the nested worker
  and reported a valid bundle satisfying its Designated Requirement.
- Info.plist `CFBundleShortVersionString` is `v0.22.42`.
- Main executable SHA-256:
  `e998e2657847caacb530a97109db8328c8179371c2a4581e2ed3399f3ba49c08`.
- Worker executable SHA-256:
  `44c5a644c988cd390e45846ba2e1a16359e13c8be9c6f81dbb9efa9a9ba2504b`.

This is an ad-hoc signed local Universal package, not a published distribution.
No user app was launched/restarted, no configuration was edited and no Terms
approval was copied. Runtime/clean-machine, original Office/PDF, resource-budget,
independent fidelity and all-OS distribution acceptance remain unfinished.
The prior interrupted 5aa package was not reused as success evidence.
