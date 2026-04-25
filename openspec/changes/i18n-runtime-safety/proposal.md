## Why

現在の i18n runtime は static JSON を型へ読み込む構造で扱いやすい一方、runtime language の異常値や format の複雑化に弱い。正式リリース後の安定性を考えると、未知の言語設定で落ちないことと、parameterized message を安全に扱う境界が必要になる。

## What Changes

- runtime language value を fallback-aware に解決し、ユーザー設定や環境由来の未知値で UI が panic しないようにする。
- 現行の `I18nOps::tf` を互換層として残しつつ、message formatting を KatanA-owned adapter へ寄せる。
- count、file count、problem total など plural-sensitive な message を棚卸しする。
- Fluent / ICU などの候補を adapter の内側で比較し、採用または明示的な延期を決める。
- locale completeness、pseudo-translation、formatter key の検査を追加する。

## Capabilities

### New Capabilities

- `i18n-runtime-safety`: i18n runtime の fallback、安全な message formatting、locale quality gate を提供する。

### Modified Capabilities

- `i18n`: runtime language lookup と parameterized message rendering の安全性を強化する。

## Impact

- `crates/katana-ui/src/i18n/*`
- `crates/katana-ui/locales/*.json`
- `crates/katana-linter` の locale AST rule
- settings persistence から読み込まれる language value
- i18n unit tests / integration tests
