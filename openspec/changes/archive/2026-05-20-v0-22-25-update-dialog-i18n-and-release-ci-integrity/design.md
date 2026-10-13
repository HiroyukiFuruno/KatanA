## Context

v0.22.24 リリース直後のユーザー検証で 3 件の欠陥が連続して報告された:

1. **Issue B**: `bump-version.sh` がローカル経路で自動付与する `[skip ci]` 文字列により、bump コミットの CI が走らず、品質ゲートが構造的に bypass されていた。ユーザーは「インチキするための罠」と評した。
2. **Issue C**: `apply_os_default_language` が first_launch 限定の sticky 仕様で、英語 OS に切り替えた後も settings に保存された "ja" がそのまま使われる。OS 追従モードが存在しない。
3. **Issue D**: update check dialog の title/body は i18n 化済みだが、error 詳細（`ureq::Error` の `Display`）は英文固定でローカライズ対象外。日本語 UI でも英文 error が混入する。

## Goals

- リリースプロセスから skip-ci の抜け道を完全に除去する（コード・script・lint で多重に保証）。
- OS locale 変更に追従できる "auto" モードを導入しつつ、明示設定モードの優先度を壊さない。
- 更新確認ダイアログのすべての文字列が、10 言語の `crates/katana-ui/locales/*.json` を介して localize される状態を構造的に保証する（AST linter で enforce）。

## Non-Goals

- ureq の error variant を逐一 i18n 化することは行うが、ureq そのものの差し替え（reqwest 等）は別 change の領域とする。
- アプリ全体の i18n リファクタリング（既存の hard-coded 文言の網羅的洗い出し）は本 change のスコープ外。本 change が触るのは update check ダイアログ周辺。
- `apply_os_default_theme` 等の OS-follow 系設定の汎用化は本 change では扱わない。language のみに絞る。

## Decisions

### B-1. `[skip ci]` の構造的除去

- `bump-version.sh` のローカルブランチ commit message を `chore: Release v${TARGET_VERSION}` に変更。`[skip ci]` 文字列は削除する。
- GitHub Actions 経由の API commit（CI コンテキスト分岐）でも `[skip ci]` を含めない。
- 新規 lint rule `no-skip-ci-marker` を `katana-linter` の `release-scripts` domain に追加し、`scripts/release/**/*.sh` および `.github/workflows/**` から `[skip ci]` / `[ci skip]` を grep ベースで検出した場合に違反とする。
- bump 後の commit が `paths-ignore` に該当しないことは、Release workflow の preflight job 内で `git diff --name-only HEAD~1` を確認し、Cargo.toml / Cargo.lock / Info.plist のいずれかが含まれる場合は CI を強制発火させる。

### B-2. 代替案: `[skip ci]` を残したまま release workflow 側で再検査

検討したが却下。bump commit が CI に乗らない状態が他の hook（i18n linter / clippy）からも見えなくなるため、別チームが将来別の bypass を加える素地を残す。**罠を作らない**方針に統一する。

### C-1. language 設定の "auto" モード

- `Settings.language` の型は `String` のままだが、`"auto"` を予約値として扱う。
- `SettingsService::resolve_effective_language(now: &dyn FnMut() -> Option<String>) -> String` を新設し、保存値が `"auto"` の場合 `OsLocaleOps::get_default_language()` を呼び、失敗時は `"en"` にフォールバック。
- `apply_os_default_language` は **first_launch 時に "auto" を書き込む** だけにする（OS 言語の即時保存はしない）。これで OS が後で変わっても自動追従する。
- `default_language()` を `"en"` から `"auto"` に変更。
- 設定 UI の Language dropdown 先頭に "自動 (OS 設定に追従)" / "Auto (follow OS)" 等を表示。各言語 JSON にラベルを追加。

### C-2. Migration

- 既存ユーザーで `language: "ja"` 等が保存されているケースは尊重する（沈黙の置換はしない）。"auto" は新規ユーザー / 明示的に選択した場合のみ。
- 設定 UI に「OS に追従に戻す」ボタンを置く（明示変更）。
- Migration test: 既存設定が読み込まれた時に値が書き換わらないことを test_load_existing_japanese_language で保証。

### D-1. `CheckUpdateError` の i18n key mapping

```rust
pub enum CheckUpdateError {
    /// Couldn't reach the server (refused, DNS, host unreachable).
    NetworkUnreachable,
    /// Connection was reset / timed out partway through.
    NetworkTimedOut,
    /// Got a non-2xx HTTP status (rate limit, server error).
    ServerStatus(u16),
    /// Proxy configuration failed (CONNECT refused, auth required, ...).
    ProxyFailed,
    /// Response body could not be decoded as the expected JSON.
    InvalidPayload,
    /// Anything else (catch-all for ureq internals we don't translate yet).
    Other(String),
}

impl CheckUpdateError {
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::NetworkUnreachable => "update_check_error_network_unreachable",
            Self::NetworkTimedOut => "update_check_error_network_timed_out",
            Self::ServerStatus(_) => "update_check_error_server_status",
            Self::ProxyFailed => "update_check_error_proxy_failed",
            Self::InvalidPayload => "update_check_error_invalid_payload",
            Self::Other(_) => "update_check_error_unknown",
        }
    }
}
```

- `state.update.check_error` の型を `Option<String>` から `Option<CheckUpdateError>` に変更。
- dialog 側は i18n bundle から localize message を取り、ServerStatus(code) のような param は `{status}` 等のプレースホルダを置換する。
- `Other(String)` も raw 文字列を表に出さず、localize した "未確認のエラー: %s" 形式で表示する（raw 部分は技術詳細としてのみ）。

### D-2. AST linter rule: `no-raw-update-check-error-display`

- `views/modals/update/**.rs` をスキャンし、`state.update.check_error` の値を **i18n bundle を介さずに `ui.label` / `format!` で表示しているケース** を検出する。
- 違反時のエラーメッセージは「`update_check_error_*` i18n key を経由して表示してください」。

### D-3. 10 言語 JSON の更新

- 既存 `crates/katana-ui/locales/*.json` (de / en / es / fr / it / ja / ko / pt / zh-CN / zh-TW) すべてに新キーを追加。
- `katana-i18n-management` skill が要求する「ja.json には日本語翻訳必須」「Linter bypass 禁止」を守る。

## Verification Strategy

- B: bump-version.sh の commit message に `skip ci` が含まれないことを test。`katana-linter` の新 rule が `[skip ci]` を検出することを test。
- C: `SettingsService::resolve_effective_language` が "auto" → OS locale → "en" の優先順位で評価されることを test。"auto" 設定下で sys-locale が "ja-JP" / "en-US" / unknown を返した場合の 3 ケースをカバー。
- D: `CheckUpdateError::from_ureq` 変換が ureq の各 variant に対して期待の variant を返すことを test。i18n 各言語で新キーが存在することを `katana-linter` の既存 `i18n-key-coverage` test 経路で保証。

## Risks

- C の migration で「既存日本語ユーザーが意図せず英語化される」事故が起きるとユーザー UX を毀損する。→ 明示テストで保証 (既存値は変更しない)。
- D の `Other(String)` で raw 英文が JA UI にじむ可能性。→ localize phrase + "詳細: …" のサンドイッチで minimize、AST linter で **raw のみの表示** を禁止。
- B の lint rule が false positive で過去の正当な `[skip ci]` 履歴コミット（仮にあれば）を弾く懸念。→ 検査対象は `scripts/release/**` と `.github/workflows/**` のみとし、git log は対象外。

## Open Questions

- "auto" モードを ON にした時、設定 UI の Language dropdown を「OS から取得した言語名 (auto)」と表示するか「Auto」だけにするか → 既存 KatanA の UX 寄せに合わせて後者を採用。
- `Other(String)` バリアントを残すか撤廃するか → 当面残す（ureq の新規 variant 追加に追従しやすい）。本 change 完了後の v0.23 系で再評価。
