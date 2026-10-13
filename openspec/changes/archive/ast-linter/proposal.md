## Why

Katanaプロジェクトには `docs/coding-rules.md` で規定された厳格なコーディング規約（i18n文字列のハードコード禁止、利用禁止型の指定など）が存在しますが、これらは現状「人間の目視レビュー」や不完全な文字列検索に依存しており、「定義はあるが守られる保証がない」状態です。
これらを機械的に強制し、偽陰性（見逃し）や偽陽性（誤検知）を撲滅するため、Rustのソースコードを抽象構文木（AST）レベルで解析するカスタムの静的解析エンジン（AST Linter）を導入します。

## What Changes

- `syn` クレートを利用した AST 解析エンジンの実装（`tests/ast_linter.rs` または専用クレート）
- `docs/coding-rules.md` に記載されたルール（特に第11章のi18n規約）をAST上のルールチェッカーとして実装
- CIおよびlefthook（`pre-commit` / `pre-push`）による `cargo test` のハードゲートにこのLinterを組み込み、規約違反コードのコミットを拒否する仕組みを構築
- UI上の単なる記号などはエラーとしない「許可リスト（Allowlist）」の機構を追加

## Capabilities

### New Capabilities

- `coding-standards-enforcement`: プロジェクト固有のコーディング規約をAST解析によりCIおよびローカルのgitフックレベルで強制する機能

### Modified Capabilities

-

## Impact

- `dev-dependencies` への依存追加（`syn`, `ignore` など）
- 開発者のローカル開発体験（違反したコードはコミットすらできなくなる強力なハードゲートの追加）
- `katana-ui` の `tests/` 下に解析エンジンファイルが追加される（将来的にワークスペース全体への拡張も可能）
