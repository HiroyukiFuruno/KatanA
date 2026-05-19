# Tasks: v0.22.25 update dialog i18n and release CI integrity - KatanA

## 要件 (Verbatim Requirements)

```text
v0.22.24 リリースで顕在化した以下 3 件の欠陥を v0.22.25 で解消する。

B. リリーススクリプト bump-version.sh が自動付与する `[skip ci]` が CI 品質
   ゲートを bypass している。これは「インチキするための罠」であり、構造的に
   除去する。

C. OS 言語追従モードがなく、一度日本語環境で起動した端末で英語 OS に切り替
   えても日本語 UI のまま (sticky)。OS に追従する "auto" モードを導入する。

D. 更新確認 dialog の error 詳細部分が i18n 化されておらず、`ureq::Error`
   の英文がそのまま表示される。`CheckUpdateError` を介して i18n key に
   mapping し、10 言語の locale JSON すべてに対応文言を追加する。
```

## 合意済み制約 (MUST)

- 既存 `language: "ja"` 等の保存値は migration で書き換えない (沈黙の上書き禁止)。
- 10 言語 (`de / en / es / fr / it / ja / ko / pt / zh-CN / zh-TW`) すべての locale JSON を同期更新する。AST linter の i18n coverage を破らない。
- `ja.json` には日本語翻訳を入れる (英語フォールバック禁止)。
- `[skip ci]` 文字列は `scripts/release/**` と `.github/workflows/**` に存在してはならない (lint で enforce)。
- `state.update.check_error` の raw 値を `ui.label` / `format!` で直接表示する経路は新 AST linter rule で禁止する。
- `apply_os_default_language` は first_launch 時に "auto" を書き込むだけにし、OS 言語の即時保存はしない。
- `release-workflow` skill / `release` skill との整合を保つ。

## Branch Rule

- ブランチ名: `release/v0.22.25-update-dialog-i18n-and-ci-integrity`
- base: `master`
- merge 先: `master` (release workflow 自動 trigger 対象)

## Release Process

1. ブランチで本タスク群を完了。
2. `scripts/release/check-pr-ready.sh 0.22.25` で機械的事前検査を通過。
3. PR を master に出し CI 通過 + マージ。
4. release workflow が自動発火（手動 dispatch にフォールバックしない）。
5. 配布バイナリで Windows 実機検証: (a) 起動時 dialog が出ない (b) "auto" 設定が OS locale を反映 (c) error 発生時に dialog 文言が完全に当該言語化されている。

## Implementation Tasks

### B. release CI integrity

- [x] B-1. `scripts/release/bump-version.sh` のローカル経路と CI 経路の双方から `[skip ci]` を撤廃する。
- [x] B-2. 撤廃を guard する `katana-linter` の新 rule `no-skip-ci-marker` を実装。`scripts/release/**/*.sh` と `.github/workflows/**` を grep ベースで検査。
- [x] B-3. ast_linter.rs の integration test に B-2 の rule を追加し、`[skip ci]` 文字列を含む fixture が違反扱いになることを保証。
- [x] B-4. Release workflow の preflight job で `git diff --name-only HEAD~1` を検査し、`Cargo.toml` / `Cargo.lock` / `Info.plist` のいずれかが差分に含まれる場合は paths-ignore を無効化して CI を強制発火させる。
- [x] B-5. bump-version.sh の unit test (またはスクリプト test) で commit message が `chore: Release v${VERSION}` 形式 (skip ci 文字列なし) であることを検証する。

#### B verification

- [x] `rg -n "\[skip ci\]|\[ci skip\]" scripts/release .github/workflows` が該当なし。
- [x] `cargo test -p katana-linter release_scripts --lib` が通過。
- [x] `cargo test -p katana-linter --test ast_linter ast_linter_release_automation_has_no_ci_bypass_markers` が通過。
- [x] `cargo test -p katana-linter --test ast_linter ast_linter_windows_msi_packaging_uses_current_version` が通過。
- [x] `bash -n scripts/release/bump-version.sh scripts/release/verify-version-bump-ci.sh` が通過。
- [x] `scripts/release/check-pr-ready.sh` が task branch / release prep 前の統合ブランチでは最終 release 検査を skip することを確認。

### C. OS-follow locale mode

- [ ] C-1. `SettingsDefaultOps::default_language` を `"en"` → `"auto"` に変更。
- [ ] C-2. `SettingsService::resolve_effective_language(detector: impl FnOnce() -> Option<String>) -> String` を新設。"auto" → OS locale → "en" の優先順位で評価する。
- [ ] C-3. `apply_os_default_language` を「first_launch 時に保存値を `"auto"` にする」だけに簡素化。OS 言語の即時保存は行わない。
- [ ] C-4. 全 caller を `resolve_effective_language` 経由に切り替え (`global_menu_context.rs`, i18n bundle ロード経路, 等)。
- [ ] C-5. 設定 UI (`crates/katana-ui/src/settings/tabs/general.rs` 付近) の Language dropdown 先頭に "Auto" を追加。各 locale JSON に label を追加。
- [ ] C-6. Migration test: 既存 `language: "ja"` の repository が `resolve_effective_language` で `"ja"` を返すこと (沈黙の置換が起きないこと) を保証。
- [ ] C-7. "auto" + OS locale が `ja-JP`/`en-US`/unknown の 3 ケースで期待値を返すユニットテスト。

### D. Update-check error i18n

- [ ] D-1. `katana_core::update` 配下に `CheckUpdateError` enum を定義 (`NetworkUnreachable / NetworkTimedOut / ServerStatus(u16) / ProxyFailed / InvalidPayload / Other(String)`)。
- [ ] D-2. `ReleaseClient::fetch_latest_release` の戻り型を `Result<Option<LatestRelease>, CheckUpdateError>` に変更し、ureq の error variant から `CheckUpdateError` への from 実装を追加。既存の anyhow::Error 経路は `From<CheckUpdateError> for anyhow::Error` で互換維持。
- [ ] D-3. `UpdateOps::check_for_updates_simple` の error 型を `String` から `CheckUpdateError` に変更し、`crates/katana-ui/src/app/update.rs` の `update_rx` / `state.update.check_error` の型をそれに合わせる。
- [ ] D-4. `views/modals/update/mod.rs` の error 表示部分を、`CheckUpdateError::i18n_key()` で i18n bundle を引いて localize された phrase を返す形に変更する。`ServerStatus(code)` のような param はプレースホルダ置換 (`{status}`)。
- [ ] D-5. 10 言語の `crates/katana-ui/locales/*.json` に以下キーを追加:
  - `update_check_error_network_unreachable`
  - `update_check_error_network_timed_out`
  - `update_check_error_server_status`
  - `update_check_error_proxy_failed`
  - `update_check_error_invalid_payload`
  - `update_check_error_unknown`
- [ ] D-6. AST linter 新 rule `no-raw-update-check-error-display` を実装。`views/modals/update/**.rs` をスキャンし `state.update.check_error` の値が i18n bundle を経由せず直接表示されているケースを違反として報告。
- [ ] D-7. integration test: 各 variant に対して dialog text が **当該 locale の文言**になることを保証。

### E. CHANGELOG and release artifacts

- [ ] E-1. `CHANGELOG.md` / `CHANGELOG.ja.md` に v0.22.25 entry を追加。Bug Fixes (B, C, D) を分けて記述。CHANGELOG.md は **英語のみ** で書き、UI 文言の引用も英語に統一する。
- [ ] E-2. `scripts/release/check-pr-ready.sh 0.22.25` を通過する。
- [ ] E-3. release notes (gh release view) と CHANGELOG.md が完全に一致することを `scripts/release/extract-notes.sh 0.22.25` で確認する。

## Done Definition

- 上記 B-1 〜 E-3 の全 task が完了し、`cargo test -p katana-core -p katana-ui -p katana-linter -p katana-platform` がローカルで通過する。
- `just check-light` が pass する。
- master へのマージ後、release workflow が **手動 dispatch なしで** 自動発火する。
- 配布バイナリの Windows 実機検証で (a) 起動時 dialog が出ない (b) "auto" モードが OS locale を反映 (c) error 発生時の dialog が完全に当該言語化されていることを確認できる。
