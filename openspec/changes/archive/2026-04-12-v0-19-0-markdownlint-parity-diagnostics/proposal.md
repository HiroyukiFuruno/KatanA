## Why

現在の diagnostics は `katana-linter` 独自ルールと簡易メッセージで構成されており、markdownlint の rule code や挙動と 1:1 では結び付いていない。そのため、ユーザーが既存の markdownlint ドキュメントや CI 結果と app 内表示を往復しづらく、今後の local LLM autofix にも標準的な入力契約が欠けている。

## What Changes

- app 内の user-facing diagnostics を markdownlint 公式 contract に合わせる
- Problems Panel で rule code、rule 名、英語説明、location、severity、参照導線を表示する
- user-facing に出荷する markdownlint diagnostics の検出挙動を official behavior に揃える
- parity が未達の internal ルールは、公式互換として見せず hidden または experimental 扱いにする
- Problems Panel から該当 location への jump、manual refresh / save refresh を整える
- future local LLM autofix で再利用できる diagnostics payload contract を定義する

## Capabilities

### New Capabilities

- `markdownlint-parity-diagnostics`: supported markdownlint rules と整合する diagnostics を app 内で実行し、Problems Panel と navigation 導線で扱う

### Modified Capabilities

## Impact

- 主な影響範囲は `crates/katana-linter/src/markdown.rs`、diagnostics rule 実装群、`crates/katana-ui/src/views/panels/problems.rs`、`crates/katana-ui/src/state/diagnostics.rs`、`crates/katana-ui/src/app/action.rs`
- official markdownlint rule metadata の管理ファイル、docs link mapping、fixture corpus が追加される可能性がある
- CI / app / future AI feature の diagnostics contract を揃える基礎になる
