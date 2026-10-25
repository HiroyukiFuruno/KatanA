# 公開macOS配布物の実測

`release-bug-triage` に従い、公開v0.22.41を実取得して検査した。
0.22.42の公開成功・全PC起動成功の証明ではない。

## 実測出力

`scripts/dev/inspect-release-asset.sh v0.22.41 macos` の該当出力:

```text
SHA-256: 33b0c079ae8e34ccd1bb0f78ab6a2304a8ada8d05358d7a891efc10c0a24ebcb
FAIL: binary architecture contract violation: KatanA Desktop.app/Contents/MacOS/KatanA architectures=['arm64'] required=['arm64', 'x86_64']
```

アプリbundle、本体、Office sidecarは存在し、ZIPの再計算checksumは公開値と一致。
本体はarm64のみで、Intelを含む現在の配布契約と不一致だった。
これはIntel起動不能の原因候補だが、報告PCのCPU/ログが未確認なので
すべての起動不能報告の原因と断定しない。

実測全文はignored `tmp/published-macos-asset-inspection-2026-10-02.log` に保持。
検査ツール自体はexit0だが、内部のarchitecture検査はFAILであり、成功扱いしない。

## 現修正の検証

既存0.22.42のpackage-macは本体とOffice sidecar双方をarm64/x86_64で
ビルド・lipo・verify_archし、ad-hoc署名する。
両CPUのコンパイル後、複数architectureを一度に渡すverify_archが
`lipo: -verify_arch requires exactly one input file` で失敗した。
入力ファイルを先に置くだけでは解消せず、各architectureを個別に検証する修正で
既存package-macがexit0になった。元実装で回帰テストRED、修正後8件GREEN。
署名のdeep/strict検査もexit0。

同bundleをworkflowと同じdittoでZIP化し、実ZIPのarchitecture検査はexit0:

```text
OK: KatanA Desktop.app/Contents/MacOS/KatanA: arm64,x86_64 minos=13.0.0
OK: KatanA Desktop.app/Contents/MacOS/kdv-office-worker: arm64,x86_64 minos=13.0.0
```

ARM実機のfresh-profile startup smokeもexit0。実行ファイルSHA-256は
`0bef445f795bd32bbd796c87f3934f2560c06614821fc4308f71d2cf630b3ed2`、
実PID44426、peak RSS239360KiB、font bytes27093388、Office workers0。
実体同一性と継続するUI heartbeatを検査し、元のメモリ基準を維持した。
Terms同意の自動化・既存同意のコピーは行っておらず、これは起動検査であって
Office機能のpackaged受入ではない。

ignoredログは`tmp/bounded-reader-package-mac-repaired-2026-10-02.log`、
`tmp/macos-universal-local-asset-contract-2026-10-02.log`、
`tmp/bounded-reader-packaged-arm64-smoke-2026-10-02.log`に保持。
x86_64のnative smokeはIntel runnerで行い、ARM hostの結果で代用しない。
公開後のZIP/DMG・checksums再取得も完了条件に残す。
