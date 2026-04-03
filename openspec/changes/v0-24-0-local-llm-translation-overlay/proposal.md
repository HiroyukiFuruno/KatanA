## Why

既存の locale JSON による UI 翻訳はあるが、markdownlint の英語説明や local LLM の生成結果など、dynamic / external English text はその仕組みだけでは翻訳できない。local LLM が有効なときは、こうした英語部分を自動表示で補助したいという要望がある。

## What Changes

- local LLM が有効な場合に、eligible な dynamic / external English text へ自動 translation overlay を適用する
- static locale JSON を置き換えず、dynamic / external text だけを対象にする
- original English text を保持したまま translated view を提供する
- translation cache と fallback を追加し、失敗時は英語表示へ戻す
- 翻訳済み overlay や非英語 text を再翻訳しない eligibility rule を追加する
- release 時点での translation target inventory を作成し、適用範囲を明示する

## Capabilities

### New Capabilities

- `dynamic-translation-overlay`: local LLM を用いて dynamic / external English text の翻訳表示を自動で重ねる

### Modified Capabilities

## Impact

- 主な影響範囲は `crates/katana-ui/src/i18n/*`、diagnostics UI、AI generation result UI、translation cache 層
- `v0.22.0` の local provider と `v0.23.0` までに増える dynamic English surface を前提にする
