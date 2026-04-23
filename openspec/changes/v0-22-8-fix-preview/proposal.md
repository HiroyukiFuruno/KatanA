## Why

現在、Problems パネルに表示される Linter の警告に対して「修正（Fix）」ボタンを押す際、実際にどのようなコード変更が行われるのかを事前に確認する手段がありません。これにより、意図しない破壊的変更やフォーマットの崩れを招く恐れがあり、ユーザーが安心して自動修正機能を利用できない状態です。修正内容を事前に確認（プレビュー）できる機能を提供することで、自動修正の安全性と UX を向上させます。

## What Changes

- Problems パネルに表示される「修正」ボタンのUIを拡張し、修正適用前に変更内容をプレビューできるようにする。
- プレビューはツールチップ（Hover）またはインラインの Diff 形式で、修正前後の差分を視覚的に分かりやすく表示する。
- LLM系の連携対応を進める前に、エディタとしての基本的な DoR（Definition of Ready）を満たすための必須要件として実装する。

## Capabilities

### New Capabilities
- `diagnostic-fix-preview`: ProblemsパネルのDiagnosticに付与されたFixの差分（Diff）を事前に確認できる機能

## Impact

- `crates/katana-ui`: Problems パネル（`diagnostics_renderer.rs` 等）の UI コンポーネントおよび描画ロジックの変更
- `crates/katana-linter`: （必要に応じて）Fix 適用前の差分生成に必要なデータの提供ロジック拡張
