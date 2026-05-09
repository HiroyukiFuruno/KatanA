## Context

KatanAではMarkdown、UI、fixture、i18n、release文書などに対して、通常の静的検査だけでは拾いにくい構造上の違反をAST lintで検知してきた。

この検査がKatanA本体に閉じたままでは、KMEやUI widgetのような新規repositoryで同じ品質基準を維持できない。

## Goals

- AST lintをP0として分離する。
- repositoryごとの独自lint driftを防ぐ。
- 違反形式、終了コード、実行入口を共通化する。
- KME、kdp、kle、kcf、kuwの着手条件へ組み込める状態にする。

## Non-Goals

- KME文書モデルをAST lint側へ持たせること。
- Rust compilerやclippyの代替を作ること。
- repository固有の全ルールを最初から共通化すること。

## Decisions

### Priority

`katana-ast-lint` はP0とする。P1 `katana-markdown-engine`、P2 `katana-ui-widget`、P3 その他より先に分離計画を固定する。

### Shared Violation Format

各repositoryは同じ違反形式を受け取る。最低限、rule id、重要度、対象file、範囲、message、修正方針を含める。

### Repo-local Adapters

repository固有のfixtureや対象file探索はadapterで持つ。共通rule本体にKatanA固有pathを直書きしない。

### KatanA Adoption Boundary

KatanA本体では既存 `crates/katana-linter` が `katana-ast-lint` と重複しやすい。取り込み時は、workspace dependencyとして `katana-ast-lint` を追加し、`just ast-lint` が外部crateのrule APIを通ることを完了条件にする。

`crates/katana-linter` を残す場合は、KatanA固有のrepository adapterまたはtest runnerに責務を限定する。共通ruleのコピー実装を残すと、分離後repositoryとKatanA本体で品質基準がずれるため、残存ruleは削除、移管、または外部crate呼び出しへ置き換える。

### No Lint Exclusion Escape

lintを通すためだけの除外設定追加は禁止する。準拠できない場合は、OpenSpecへ例外理由と代替案を明文化してから判断する。

## Risks

- KatanA固有pathを共通ruleへ混ぜると、新規repositoryで使えない。
- 先にKME実装へ入ると、KME側に一時的な独自lintが増え、後から統一しにくくなる。
