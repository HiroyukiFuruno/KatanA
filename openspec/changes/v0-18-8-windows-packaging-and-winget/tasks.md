## Branch Rule

本タスクでは、ユーザーの指定に基づき以下のブランチ運用を厳格に適用します：

- **統合（Base）ブランチ**: `release/v0.18.8`
- **各タスクの作業ブランチ**: `release/v0.18.8-task-x` (xはタスク番号)

各タスクの実装開始前に、`release/v0.18.8` から `release/v0.18.8-task-x` を作成して作業してください。
実装完了後は `/openspec-delivery` を使用して統合ブランチ（`release/v0.18.8`）へPRを作成・マージしてください。

## 1. Winget Bootstrap / Update Flow Separation

- [x] 1.1 `scripts/release/sync-external.sh` と `.github/workflows/build-and-release.yml` を見直し、少なくとも `WINGET_GH_TOKEN (classic PAT with public_repo) 有無` → `Windows MSI artifact 有無` → `komac 有無` → `komac list HiroyukiFuruno.katana-desktop 成否` の順で分岐し、upstream 未存在の間は `komac update` を実行せず bootstrap path が必要だと明示する。`github.token` fallback は残さない
- [x] 1.2 初回再申請用の winget submit 導線を明確化する。`komac new` は non-TTY CI で使えない前提とし、manifest file を生成して `komac submit --yes --token "${WINGET_GH_TOKEN}"` する path、または maintainer local TTY での bootstrap path のどちらかに固定する
- [x] 1.3 package 未存在時の skip message と、package 既存在時の `komac update` path が release log 上で識別できることを確認する

### Definition of Done (DoD)

- [x] 初回再申請と将来の version update の flow が混同されていない
- [x] winget PR 作成用 token source が `WINGET_GH_TOKEN` に固定され、`github.token` fallback が除去されている
- [x] CI と local helper が同じ Windows artifact 名と publish URL 契約を使っている

## 2. Windows Packaging Self-Containment

- [x] 2.1 Windows binary の VC++ runtime 依存を build policy 側で解消する。`x86_64-pc-windows-msvc` 向けに `crt-static` を有効化し、manifest dependency を消すだけの対症療法にしない
- [x] 2.2 Windows runner 上で packaged `KatanA.exe` の import table を確認し、`VCRUNTIME140*.dll` / `api-ms-win-crt-*` 依存が除去されたことを検証する
- [x] 2.3 `komac analyze` / generated manifest review により、`KatanA-windows-x86_64.msi` が `Scope: user` / `InstallerType: wix` を維持しつつ、problematic な `Dependencies: Microsoft.VCRedist.2015+.x64` を含まないことを確認する
- [x] 2.4 `README.md` / `CHANGELOG.md` / release note に、Windows 配布形式と install prerequisites の実態が一致していることを確認する

### Definition of Done (DoD)

- [x] `KatanA-windows-x86_64.msi` / `KatanA.exe` が VC++ runtime 外部依存なしで配布できる
- [x] import table, MSI metadata, generated manifest の確認証跡が揃っている

## 3. Windows Installer UX Refresh

- [ ] 3.1 `crates/katana-ui/wix/main.wxs` の current flow (`WixUI_FeatureTree`) は維持したまま、初回導入時の見た目と文言を KatanA 向けに整理する
- [ ] 3.2 installer metadata（Product 名、説明、ARP 表示、Feature 名）を更新し、古い印象を与える既定表現を除去する
- [ ] 3.3 `WixUIBannerBmp` / `WixUIDialogBmp` を追加し、`Product.ico` と整合する branding asset を適用する
- [ ] 3.4 Windows installer 画面の確認証跡を取得し、`v0.18.8` の申請時に参照できる状態にする

### Definition of Done (DoD)

- [ ] installer 画面が KatanA branding と整合し、既定の古い WiX 画面に見えにくい状態になっている
- [ ] Windows 向け install 導線のスクリーンショットまたは同等の証跡が残っている

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Confirm Windows release artifacts and GitHub Release asset URLs for `v0.18.8`
- [ ] 4.4 Run `make release VERSION=0.18.8` (locally) and update CHANGELOG (`changelog-writing` skill)
- [ ] 4.5 Create PR targeting `master` and ensure `Release Readiness` CI matches (PR merge will auto-trigger release)
- [ ] 4.6 Verify `scripts/release/sync-external.sh` does not silently fail for `HiroyukiFuruno.katana-desktop`, that the chosen bootstrap / update path for `v0.18.8` is documented and reproducible, and that winget sync no longer relies on `github.token` fallback
