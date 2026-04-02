## Why

Markdown 文書が増えると、ファイル名検索だけでは目的の情報に辿り着けません。本文中の見出し、段落、コード断片を直接探せる検索が必要です。
既存の workspace file search は「どのファイルを開くか」に最適化されているため、「どこにその記述があるか」を探す体験を補完する変更として今導入します。

## What Changes

- Markdown 文書の内容を対象にした本文検索を追加する
- 検索語に一致する本文スニペットを、一致位置ごとに結果として表示する
- 検索結果から該当 Markdown ファイルを開き、ヒット箇所へ移動できるようにする
- 検索語を editor/code view と preview の両方でハイライトする
- 検索語の履歴保存、履歴クリア、履歴再利用を初版に含める
- 現在の検索語に対する next / previous match ジャンプを初版に含める
- 対象範囲や上限付き結果表示など、重いワークスペースでも使える検索体験を定義する

## Capabilities

### New Capabilities

- `markdown-content-search`: 開いている workspace 内の Markdown 文書本文を検索し、結果から該当箇所へ到達できる機能

### Modified Capabilities

## Impact

- `crates/katana-ui`: 検索 UI、結果表示、editor/preview highlight、該当箇所へのナビゲーション
- `crates/katana-core` または検索インデックス層: Markdown 文書本文のインデックスと照合
- `crates/katana-platform`: 検索履歴、検索対象、結果上限などの設定や永続化が必要なら拡張
- `openspec/specs`: 新規 capability spec と既存検索 spec の役割整理
