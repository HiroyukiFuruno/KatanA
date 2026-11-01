# v0.22.45 配布アセット実測

実行: `scripts/dev/inspect-release-asset.sh v0.22.45 macos`。以下は実出力（色指定のみ除去）。

```text
[INFO]  Repo: HiroyukiFuruno/KatanA
[INFO]  Tag : v0.22.45

== KatanA-macOS.zip (zip) ==
[INFO]  Downloading KatanA-macOS.zip ...
[OK]    Downloaded KatanA-macOS.zip (225M)
SHA-256: e3de80920177e627674906356e5e86090764a250b93b528d80fc12258a9d2954
--- entries (head 50) ---
KatanA Desktop.app/
KatanA Desktop.app/Contents/
KatanA Desktop.app/Contents/_CodeSignature/
KatanA Desktop.app/Contents/_CodeSignature/CodeResources
KatanA Desktop.app/Contents/MacOS/
KatanA Desktop.app/Contents/MacOS/kdv-office-worker
KatanA Desktop.app/Contents/MacOS/KatanA
KatanA Desktop.app/Contents/Resources/
KatanA Desktop.app/Contents/Resources/icon.icns
KatanA Desktop.app/Contents/Info.plist

[OK]    Asset contract OK: 'KatanA Desktop.app/' present
[OK]    Asset contract OK: 'KatanA Desktop.app/Contents/MacOS/kdv-office-worker' present
OK: KatanA Desktop.app/Contents/MacOS/KatanA: arm64,x86_64 minos=13.0.0
OK: KatanA Desktop.app/Contents/MacOS/kdv-office-worker: arm64,x86_64 minos=13.0.0
[OK]    Binary architecture contract satisfied

== checksums.txt verification ==
[INFO]  Downloading checksums.txt ...
[OK]    Downloaded checksums.txt (4.0K)
e3de80920177e627674906356e5e86090764a250b93b528d80fc12258a9d2954  KatanA-macOS.zip
77f5da0da076e8376d8e5066ca219eecc2d45b772cb122bb0c406c52b46eafa7  KatanA-Desktop-0.22.45.dmg
9061d1f98fee376fd596fa2d6adb4e5d53d1dca3d05acce2d10f5092603c0510  KatanA-linux-x86_64.tar.gz
fbeb0791906bfc8e583b367a155c99d65e1cacfed8ec8f30d74d2c7ce1eb3571  KatanA-windows-x86_64.zip
8249111dfb3e74d101840c2dbbae61709bb181122a1b98c14663927d272339cc  KatanA-windows-x86_64.msi
--- recomputed ---
[OK]    KatanA-macOS.zip matches published checksum

== Summary ==
[OK]    Inspection complete. Use this output as the verification baseline.

Expected asset contract:
  Linux  tar.gz: top-level files 'KatanA', 'kdv-office-worker'
  Win    zip   : top-level files 'KatanA.exe', 'kdv-office-worker.exe'
  macOS  zip   : app bundle with 'KatanA' and 'kdv-office-worker'

If actual entries diverge from the contract, file it under
  scripts/release/check-release-asset-contract.sh
or fix the packaging scripts (scripts/build/package-*.sh).

```
