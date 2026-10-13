## Why

Windows 配布と winget 再申請の作業は、UI 改善中心の `v0.18.7` change とは性質が異なる。現状の課題は release helper、Windows binary の runtime 依存、WiX installer UX、winget 初回申請フローにまたがっており、別 change として `v0.18.8` に切り出した方が実装責務と検証観点が明確になる。

## What Changes

- **winget submit flow の分離**:
  - 初回申請 bootstrap path と既存 package への `komac update` flow を区別する。
  - `komac` に渡す token source を `WINGET_GH_TOKEN` に固定し、`github.token` fallback を除去する。
- **Windows binary の self-contained 化**:
  - `x86_64-pc-windows-msvc` 向けに `crt-static` を適用し、VC++ runtime 外部依存を配布物から除去する。
- **installer metadata / branding の更新**:
  - `WixUI_FeatureTree` は維持しつつ、文言、ARP 表示、banner/dialog bitmap を KatanA 向けに整理する。
- **release verification の明確化**:
  - import table、`komac analyze`、generated manifest、release asset URL、installer screenshot を `v0.18.8` の完了条件として明示する。

## Capabilities

### Modified Capabilities

- `desktop-release-distribution`: Windows `.msi` / `.zip` 成果物、winget 初回申請と version update の分岐、release helper の token 契約、WiX installer branding を `v0.18.8` の配布方針に合わせて整理する。

## Impact

- `scripts/release/*`: winget helper と bootstrap/update 分岐への影響。
- `.github/workflows/build-and-release.yml`: winget token 注入と publish flow への影響。
- `.cargo/config.toml` または同等の Windows build policy: `crt-static` 適用への影響。
- `crates/katana-ui/wix/main.wxs`: installer metadata と branding asset への影響。
- `README.md`, `CHANGELOG.md`, release notes: Windows 配布前提の記述整合への影響。
