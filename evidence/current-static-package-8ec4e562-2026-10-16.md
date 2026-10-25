# Static macOS package verification at 8ec4e562

## Source and execution

- Source: `8ec4e5624a5c2d7acc948b3df87a44422720d692`, clean at both source-identity observations.
- Official command: `just package-mac`; session 31345 returned exit 0.
- Raw build: `tmp/8ec4e562-package-mac-2026-10-16.log`.
- Source identity: `tmp/8ec4e562-package-source-2026-10-16.json`; begin and verify both returned exit 0.
- Root lock SHA-256: `a6035ec9364f734fd132d58ca909cfed0b9c9e13c25ffc894165d79060db2af4`.
- Screenshot lock SHA-256: `ef7151771258e762d1962647ccf4625dd668d48ec3a5246b60bcb48734955896`.
- Shared build capacity was coordinated with the existing KDV task. No app launch or settings/Terms operation was performed.

## Actual bundle inspection

Bundle: `target/release/bundle/osx/KatanA Desktop.app`.

`file` and `xcrun vtool -show-build` inspected both actual executable names, `KatanA` and `kdv-office-worker`. Each contains x86_64 and arm64 slices; all four slices declare minimum macOS 13.0. The initial inspection used an incorrect worker filename; that lookup failed and was not accepted as verification. The corrected inspection uses the actual directory entries.

`codesign --verify --deep --strict --verbose=2` succeeded. `codesign -dv --verbose=2` reports `Signature=adhoc` and no paid TeamIdentifier. Info.plist declares `v0.22.42`.

| File | SHA-256 |
| --- | --- |
| `Contents/MacOS/KatanA` | `054d511d4bab7b3a613852843ec2bcbb00442070a20817b42092a28cfc5508e8` |
| `Contents/MacOS/kdv-office-worker` | `835ce401af1335ac86ef68f40af702e9ae2cc18bdb9ed702cad8cf1ed53bc771` |
| `Contents/Info.plist` | `8c7fbdb0a3cbb2fd27a9582950537afa4cdbbc8a0393da382c56c4a85691339c` |

Raw inspection: `tmp/8ec4e562-package-static-inspection-2026-10-16.log`.

## Remaining acceptance

This proves a source-bound local package build and static macOS binary/signature properties only. It does not prove clean-machine execution, Intel execution, Windows/Linux execution, original-document host acceptance, startup-failure resolution, human approval, current/future-HEAD required CI, all five public assets or a GitHub Release. Those gates remain unchanged. Later source/CI changes do not inherit an assertion that they were built by this command.
