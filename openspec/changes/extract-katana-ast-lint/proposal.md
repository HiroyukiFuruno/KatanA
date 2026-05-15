## Why

KMM、preview、editor、export、widgetを別repositoryへ分離すると、抽象構文木検査（AST lint）のルール、実行入口、違反形式がrepositoryごとにずれる。

KMMより先に `katana-ast-lint` を分離し、KatanA ecosystem全体で同じ品質ゲートを使える状態にする。

## What Changes

- `katana-ast-lint` をP0 repositoryとして分離する
- KatanA本体にあるAST lint相当の検査、ルール、出力形式を棚卸しする
- 各repositoryから同じ実行入口と同じ違反形式を参照できるようにする
- KMM以降の分離repositoryは、この共通AST lintを着手条件（DoR: Definition of Ready）に含める

## Capabilities

### New Capabilities

- `shared-ast-lint-governance`: 分離repository横断のAST lint統制を定義する

## Impact

- `katana-ast-lint`: 共通AST lint本体の新規repository
- `katana`: 既存AST lint相当の検査棚卸しと移行元
- `katana-markdown-model`: P1以降の品質ゲート利用者
- `katana-document-preview`, `katana-language-editor`, `katana-diagram-renderer`, `katana-canvas-forge`, `katana-ui-widget`: 後続利用者
