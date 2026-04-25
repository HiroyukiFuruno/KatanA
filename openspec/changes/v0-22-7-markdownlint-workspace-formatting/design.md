## Context

現在の図形描画プレビューは ` ``` ` フェンスを直接文字列で見ている。`crates/katana-core/src/preview/section/fence.rs` と `crates/katana-core/src/markdown/fence/mod.rs` の両方にバッククォート前提の処理があり、`~~~mermaid` のようなチルダフェンスは図形として抽出されない。

Lint 設定は `LinterSettings` に `enabled`、`use_workspace_local_config`、`rule_severity` がある。現在の `MarkdownLinterBridgeOps::evaluate_document` は KML の設定ファイル由来の `LintOptions` を読み込んだ後、KatanA 側の `rule_severity` を上書きする。このため「ワークスペース設定を使う」と「KatanA 側で警告/エラー/無視を選ぶ」の責務が混ざり、`.markdownlint.json` の内容が効いているように見えにくい。

エクスプローラーは、ディレクトリとファイルの行にコンテキストメニューがある一方、空き領域には操作入口がない。新規作成の `AppAction::RequestNewFile` / `RequestNewDirectory` は既にあるため、空き領域とヘッダーアイコンは同じ action に接続できる。KML のフォーマット機能はまだアプリ内 action として公開されていないため、ファイル単位・ワークスペース単位の action 境界を追加する。

## Goals / Non-Goals

**Goals:**

- 図形描画は ` ``` ` と `~~~` の両方で同じレンダリング結果になる。
- Lint の一般設定は、単純なオン/オフと「無視 / 警告 / エラー」の選択に限定する。
- Lint の詳細設定は `.markdownlint.json` / `.markdownlint.jsonc` の内容に対する上乗せ編集として扱う。
- グローバル設定とワークスペース設定を分け、ワークスペース設定を優先する。
- 一般設定で「無視」にした場合でも、詳細設定の履歴を失わない。
- KML へ渡す設定は、実際に有効な `.markdownlint.json` と KatanA 側の重大度設定を統合した結果になる。
- ファイル単位、ワークスペース単位の Markdown フォーマット操作を UI から実行できる。
- ファイル作成・フォルダ作成は、エクスプローラーのヘッダーと空き領域から起動できる。
- 追加アイコンは各アイコンパック固有の SVG を使う。

**Non-Goals:**

- Lint ルール自体の新規実装。
- `.markdownlint.json` 以外の独自ファイルをワークスペース直下の主要設定保存先にすること。
- フォーマット結果の差分プレビュー機能。
- LLM（local LLM）による自動修正。
- エクスプローラー全体のデザイン刷新。

## Decisions

### 1. 図形フェンスは fence token を抽象化して扱う

` ``` ` 専用の処理を、バッククォートとチルダの両方を表す小さな値に置き換える。開始フェンスと終了フェンスは同じ文字種である必要があり、終了フェンスは開始フェンス以上の長さを許容する。これは CommonMark の fenced code block の挙動に合わせる。

採用理由:

- `~~~` だけを追加する条件分岐より、既存のネスト済みフェンス判定を壊しにくい。
- プレビュー分割と HTML エクスポートの両方へ同じ考え方を適用できる。

### 2. 一般設定の重大度と `.markdownlint.json` のルール適用を分離する

KatanA の一般設定は `RuleSeverity` を使い、表示上の「無視 / 警告 / エラー」を決める。ただし「無視」は KML のルール設定を即時に消す意味ではなく、KatanA 側のワークスペース設定として診断表示から除外する意味にする。

`.markdownlint.json` はルールの有効/無効、詳細値、`default` を持つ。一般設定で「無視」以外へ戻した場合は、保持していた詳細設定を `.markdownlint.json` に戻し、ルール適用を `false` のまま放置しない。

採用理由:

- ユーザーが「警告/エラーの見え方」を変える操作と、`.markdownlint.json` の実体を書き換える操作を分けられる。
- 高度な設定を一時的に無効化しても、再度有効化した時に履歴を復元できる。

### 3. 設定の優先順位は workspace > global > default とする

グローバル設定は `~/Library/Application Support/KatanA/.markdownlint.json` などの OS 設定領域に置く。ワークスペース設定はワークスペース直下の `.markdownlint.json` または `.markdownlint.jsonc` を優先する。

ワークスペース単位の KatanA 重大度設定は、ワークスペースごとに復元できる必要がある。ただし、`.markdownlint.json` は markdownlint 互換の外部設定ファイルであり、KatanA 独自の重大度情報をそのまま混ぜると外部 linter との互換性が崩れる。実装では次のどちらかを選び、選んだ理由をコード近くのテストで固定する。

- `.markdownlint.json` の標準外キーを KML が安全に無視できる場合は、KatanA 用の namespace を設けて同ファイルへ保存する。
- 標準外キーが危険な場合は、既存のワークスペース状態保存先に重大度だけを保持し、`.markdownlint.json` はルール適用・詳細値だけに限定する。

採用理由:

- ユーザー要件の「ワークスペースごとに設定を分ける」と、外部 linter 互換性の両方を守るため。
- KML 側の config 読み込み仕様に依存しすぎない。

### 4. KML へ渡す設定は KatanA 側で有効設定へ正規化する

KML API が `--config` 相当のファイルパス入力を提供している場合は、それを使う。API がファイルパスを受け取れない場合は、KatanA 側で `.markdownlint.json` / `.markdownlint.jsonc` を読み込み、KML の `LintOptions` または同等の構造へ変換して渡す。

採用理由:

- CLI では `--config` が使えても、ライブラリ API で同じ入口があるとは限らない。
- 設定が効かない不具合を「読み込み漏れ」か「優先順位の上書き」かに分解してテストできる。

### 5. フォーマット操作は action とサービスで分ける

UI は `FormatMarkdownFile(path)` と `FormatWorkspaceMarkdown(root)` のような action だけを発行する。実際の KML 呼び出し、対象 Markdown ファイル列挙、保存、診断再実行、エラー表示は専用サービスに閉じる。

採用理由:

- ファイル行、空き領域、将来のショートカットから同じ処理を呼べる。
- ワークスペース一括フォーマットで UI とファイル走査が密結合しない。

### 6. エクスプローラーの追加アイコンは icon pack ごとに取得する

画像の見た目は「紙アイコンに小さなプラス」「フォルダアイコンに小さなプラス」である。実装では `katana-icon-management` に従い、`scripts/download_icon.sh` で各アイコンパック固有の `file-plus` / `folder-plus` 相当を取得し、`Icon::FilePlus` / `Icon::FolderPlus` として登録する。既存 SVG のコピーで代用しない。

採用理由:

- アイコンパックごとの見た目の一貫性を守れる。
- KatanA 固有アイコンだけに偏らない。

## UI Notes

画面上では、エクスプローラーのフィルターアイコンの左に、小さなファイル追加アイコンとフォルダ追加アイコンが並ぶ。クリックすると、現在のワークスペース直下を親としてファイル作成/フォルダ作成の入力画面が開く。

ファイル行を右クリックした時は、対象が有効な Markdown ファイルなら「ファイルをフォーマットする」が表示される。エクスプローラーのファイルやフォルダがない空白部分を右クリックした時は、「ワークスペース内の Markdown を一括フォーマット」「ファイルの新規作成」「フォルダの新規作成」が表示される。

Lint 設定画面では、「ワークスペース固有の `.markdownlint.json` を使用する」を切り替えても、画面は一般設定のまま留まる。高度な設定はユーザーが明示的に開いた時だけ表示される。

## Risks / Trade-offs

- **[Risk] `.markdownlint.json` に KatanA 独自設定を混ぜると、外部 linter が拒否する可能性がある**
  - Mitigation: KML と markdownlint 互換性をテストし、危険な場合は重大度だけ既存 workspace state に保存する。
- **[Risk] 一般設定の「無視」と `.markdownlint.json` の `false` が二重管理に見える**
  - Mitigation: UI 文言で「診断で無視」と「ルール自体を無効」を分け、詳細設定への導線を明示する。
- **[Risk] ワークスペース一括フォーマットが大きな差分を生む**
  - Mitigation: 対象は Markdown ファイルに限定し、実行後に変更ファイル数と失敗ファイルをステータス表示する。
- **[Risk] `~~~` 対応で既存のネスト済みコードブロック回避が壊れる**
  - Mitigation: バッククォートとチルダ双方で、ネスト済み diagram が誤抽出されない回帰テストを追加する。

## Migration Plan

1. `~~~mermaid`、`~~~plantuml`、`~~~drawio` の失敗テストを追加する。
2. フェンス抽出をバッククォート/チルダ両対応にする。
3. Lint 設定の責務を一般設定、詳細設定、グローバル設定、ワークスペース設定へ分ける。
4. KML に渡す effective config をテストで固定する。
5. ファイル単位のフォーマット action とサービスを追加する。
6. ワークスペース一括フォーマット action とサービスを追加する。
7. エクスプローラーのファイル行と空き領域へコンテキストメニューを追加する。
8. エクスプローラーヘッダーへファイル追加/フォルダ追加アイコンを追加する。
9. 追加アイコンを全アイコンパックに登録する。
10. UI スナップショットと `make check` で検証する。

## Open Questions

- KML の現在バージョンで format API がファイルパス、文字列、設定構造体のどれを受け取るか。
- `.markdownlint.json` に KatanA namespace を保存しても KML と外部 markdownlint が安全に無視できるか。
