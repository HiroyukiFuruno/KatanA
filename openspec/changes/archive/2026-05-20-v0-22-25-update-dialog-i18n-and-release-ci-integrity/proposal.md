## Why

KatanA v0.22.24 リリースを観察した結果、以下 3 つの regression / 品質欠陥がユーザー検証で表面化した。いずれも次バージョン v0.22.25 までに解消し、リリースゲートの抜け道とユーザー UX の不整合を構造的に閉じる必要がある。

### B. リリース手順における CI 品質ゲートの bypass

`scripts/release/bump-version.sh` がローカル実行時に自動付与する `chore: Release v${TARGET_VERSION} [skip ci]` commit が、後続の CI ジョブ（lint / test / linter / Linuxbrew formula 検査）を**スキップさせる罠**になっている。bump 自体は Cargo.toml / Cargo.lock / Info.plist の差分だけだが、master に乗ったあと release workflow がそれを基にビルドするため、結果として「品質ゲートを通っていないコミットがリリース artifact に含まれる」状態が常態化していた。

ユーザーから「お前らがインチキするために勝手に仕込んだ罠だ」と明示的に指摘された。意図せざる skip-ci 経路は構造的に除去する。

### C. OS ロケール追従モードの欠如（sticky 言語）

`apply_os_default_language` は first_launch 時のみ OS locale を設定に反映し、二回目以降は settings 永続化された値を使う sticky 仕様。一度日本語 OS で起動して `language: "ja"` が persist されると、その後 OS を英語に切り替えても UI は日本語のまま。Windows で OS を英語表示にしたユーザーが「英語設定で日本語表示してどうするんだよ」と指摘した root cause。

OS 言語に追従するモード（`auto`）を導入し、明示的な手動設定がない限り起動毎に OS locale を再評価する。

### D. 更新確認ダイアログの error 詳細部分が i18n 化されていない

`crates/katana-ui/src/views/modals/update/mod.rs` の error dialog は title / body を i18n キー (`failed_to_check` 等) で localize しているが、error 内容（`ureq::Error` の文字列）は **英語固定**で UI に流し込まれる。日本語 UI で「io: Connection refused」がそのまま表示されるなど、i18n が分断されていた。

ureq の error 種別を i18n キーに mapping し、10 言語 (de / en / es / fr / it / ja / ko / pt / zh-CN / zh-TW) すべての locale JSON に対応する文言を追加する。

## What Changes

### B. release CI integrity

- `scripts/release/bump-version.sh` から `[skip ci]` 自動付与を撤廃。bump コミットも CI に通す。
- `lefthook.yml` / Release workflow の trigger を見直し、`paths-ignore` の境界条件で bump コミット相当の差分が品質ゲートを通ることを保証する。
- AST linter / scripts linter に「`[skip ci]` を含む release script はリポジトリ内に存在しない」ことを検査する rule を追加する。

### C. OS-follow locale mode

- `SettingsService::language` に "auto"（OS 追従）と明示言語コード（"en" / "ja" / ...）の 2 モードを許容する。
- "auto" 設定時は起動毎に `OsLocaleOps::get_default_language()` を再評価する。
- 設定 UI（Settings → General → Language）に "OS に追従 (auto)" 選択肢を追加し、10 言語 JSON にも label を入れる。
- `default_language()` のデフォルト値を `"en"` から `"auto"` に切り替える。

### D. Update-check error i18n

- `katana_core::update::CheckUpdateError` を整え、ureq の error variant（io / proxy / status / decode 等）を **i18n キーに mapping** する。
- modals/update/mod.rs の error 表示パスを `(localized_phrase, optional raw detail)` の組に変える。
- 10 locale JSON に `update_check_error_io_connection_refused`, `update_check_error_io_timed_out`, `update_check_error_proxy_failed`, `update_check_error_invalid_payload`, `update_check_error_unknown` を追加。
- 追加の AST linter rule: `views/modals/update/` 配下で `state.update.check_error` を **i18n を介さずに直接表示しない** ことを enforce。

## Capabilities

### New Capabilities

- `update-check-localization`: 更新確認ダイアログのすべての文字列（title / body / 詳細 error）が、OS locale 設定および明示的言語設定の両モードで一貫して localize される capability。
- `release-ci-integrity`: リリーススクリプトおよびリリースフローが、`[skip ci]` 等の自動 bypass 手段を持たず、master に乗るすべての commit が品質ゲートを通ることを保証する capability。

### Modified Capabilities

- 既存 `katana-i18n-management` skill の expectations と整合させる（10 locale JSON 同期、ja.json への日本語翻訳必須 など）。

## Impact

- `crates/katana-platform`: SettingsService の language 解決、`OsLocaleOps` の呼び出し点。
- `crates/katana-core/src/update`: ureq error → i18n key mapping のために `CheckUpdateError` を導入し、`fetch_latest_release` の戻り型を変更（後方互換は anyhow::Error への conversion を保持）。
- `crates/katana-ui/src/views/modals/update`: error 表示の renderer を localize 経路へ。
- `crates/katana-ui/locales/*.json` (10 ファイル): error mapping 用キー追加 + "auto" 言語 label。
- `crates/katana-linter`: skip-ci 検査 rule, `state.update.check_error` 直接表示禁止 rule。
- `scripts/release/bump-version.sh`: `[skip ci]` 撤廃 + 撤廃を検出する regression test。
- CHANGELOG.md / CHANGELOG.ja.md: v0.22.25 entry。
