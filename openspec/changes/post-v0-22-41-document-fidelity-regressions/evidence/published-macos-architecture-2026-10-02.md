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
ビルド・lipo・verify_archし、ad-hoc署名する。現在その既存ターゲットを実行中。
両architectureの実体確認、署名確認、arm64 fresh-profile起動smokeが必要。
x86_64のnative smokeはIntel runnerで行い、ARM hostの結果で代用しない。
公開後のZIP/DMG・checksums再取得も完了条件に残す。
