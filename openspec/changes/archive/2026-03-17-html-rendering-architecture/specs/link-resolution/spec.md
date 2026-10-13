## ADDED Requirements

### Requirement: LinkTarget はリンク先を3種に分類する

`LinkTarget::resolve()` は href 文字列と base_dir を受け取り、リンク先を External, InternalFile, Anchor の3種に分類しなければならない（SHALL）。

#### Scenario: HTTP リンクの分類

- **WHEN** `LinkTarget::resolve("<https://github.com/org/repo">, base_dir)` を呼んだ場合
- **THEN** `LinkTarget::External("<https://github.com/org/repo")`> を返す

#### Scenario: HTTPS リンクの分類

- **WHEN** `LinkTarget::resolve("<http://example.com">, base_dir)` を呼んだ場合
- **THEN** `LinkTarget::External("<http://example.com")`> を返す

#### Scenario: 相対パスリンクの分類

- **WHEN** `LinkTarget::resolve("README.ja.md", Path::new("/project"))` を呼んだ場合
- **THEN** `LinkTarget::InternalFile(PathBuf::from("/project/README.ja.md"))` を返す

#### Scenario: アンカーリンクの分類

- **WHEN** `LinkTarget::resolve("#installation", base_dir)` を呼んだ場合
- **THEN** `LinkTarget::Anchor("installation")` を返す

### Requirement: LinkTarget はデフォルトのリンクアクションを返す

各 `LinkTarget` バリアントは `default_action()` メソッドで適切な `LinkAction` を返さなければならない（SHALL）。

#### Scenario: 外部リンクはブラウザで開く

- **WHEN** `LinkTarget::External(url).default_action()` を呼んだ場合
- **THEN** `LinkAction::OpenInBrowser` を返す

#### Scenario: 内部ファイルリンクは現在のタブでナビゲートする

- **WHEN** `LinkTarget::InternalFile(path).default_action()` を呼んだ場合
- **THEN** `LinkAction::NavigateCurrentTab` を返す

#### Scenario: アンカーリンクは現在のタブでナビゲートする

- **WHEN** `LinkTarget::Anchor(id).default_action()` を呼んだ場合
- **THEN** `LinkAction::NavigateCurrentTab` を返す

### Requirement: Markdown リンクと HTML リンクの両方を統一的に処理する

パーサーは Markdown 記法 `[text](url)` と HTML 記法 `<a href="url">text</a>` の両方を `HtmlNode::Link` として同一の `LinkTarget` モデルに変換しなければならない（SHALL）。

#### Scenario: Markdown リンクの統一処理

- **WHEN** `[日本語](README.ja.md)` を含むテキストをパースした場合
- **THEN** `HtmlNode::Link { target: InternalFile("README.ja.md"), children: [Text("日本語")] }` を生成する

#### Scenario: HTML リンクの統一処理

- **WHEN** `<a href="<https://github.com">>GitHub</a>` を含む HTML をパースした場合
- **THEN** `HtmlNode::Link { target: External("<https://github.com")>, children: [Text("GitHub")] }` を生成する

### Requirement: LinkAction は将来のナビゲーション拡張に対応できる

`LinkAction` enum はタブ管理やヒストリーナビゲーションの拡張ポイントとして設計しなければならない（SHALL）。初期実装では `OpenInBrowser`, `NavigateCurrentTab`, `OpenInNewTab` の3種を定義する。

#### Scenario: 将来の拡張に対応するenum設計

- **WHEN** 新しいナビゲーション方式（例: 分割ビューで開く）を追加する場合
- **THEN** `LinkAction` enum にバリアントを追加するだけで、既存の match 文がコンパイルエラーで漏れを検出する
