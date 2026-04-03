## Why

現在の shortcut は `shell_ui.rs` に hard-coded されており、変更や確認の導線がない。全動作へ keyboard access を広げるには、command inventory と settings persistence を前提にした configurable shortcut system が必要である。

## What Changes

- user-configurable shortcut system を導入する
- default shortcuts を command inventory に紐付けて定義する
- settings UI から shortcut を変更できるようにする
- 既存に割り当て済みの shortcut を重複登録できないようにする
- 衝突時は、何に割り当て済みかを popup で表示する
- restore default と platform-aware modifier handling を提供する

## Capabilities

### New Capabilities

- `shortcut-customization`: command ごとの shortcut を表示、変更、検証、永続化できる

### Modified Capabilities

## Impact

- 主な影響範囲は `crates/katana-ui/src/shell_ui.rs`、`crates/katana-platform/src/settings/*`、settings UI、future command inventory
- `v0.19.0` の shared command inventory を前提とする
