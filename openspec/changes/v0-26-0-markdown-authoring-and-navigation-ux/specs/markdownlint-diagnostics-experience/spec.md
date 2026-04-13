## ADDED Requirements

### Requirement: user-facing markdownlint diagnostics は shipped supported rule set を横断して評価されなければならない

システムは、user-facing に出荷する markdownlint diagnostics について、単一 rule のみではなく shipped supported rule set 全体を評価し、検出できた violation を同一 contract で扱わなければならない（SHALL）。

#### Scenario: 複数の supported rule violation を同時に検出する

- **WHEN** active Markdown document が shipped supported rule set に含まれる複数 rule に違反している
- **THEN** system は各 violation を official rule code 単位で diagnostics payload に含める
- **THEN** system は MD001 のみを特別扱いせず、同一 surface contract で Problems Panel と editor inline surface へ流す

### Requirement: editor は diagnostics location に inline decoration を表示しなければならない

システムは、editor 上で表示可能な markdownlint diagnostic location について、該当範囲に warning decoration を表示しなければならない（SHALL）。

#### Scenario: offending range に underline を表示する

- **WHEN** active editor buffer に markdownlint diagnostic が存在し、line / range が現在の buffer に解決できる
- **THEN** system は該当範囲に inline warning decoration を表示する
- **THEN** decoration は Problems Panel の diagnostic item と同じ rule code に対応付けられる

#### Scenario: theme override が decoration color に反映される

- **WHEN** user が theme settings で diagnostics decoration color を変更する
- **THEN** editor 上の warning decoration color は再起動なしで更新される

### Requirement: inline diagnostics から popup detail を開けなければならない

システムは、editor 上の warning decoration から official rule detail、message、docs link、available action を確認できる popup を表示しなければならない（SHALL）。

#### Scenario: decoration hover で diagnostic popup を表示する

- **WHEN** user が inline warning decoration を hover または focus する
- **THEN** system は official rule code と explanation を含む popup を表示する
- **THEN** popup から docs link または同等の詳細導線へ到達できる

### Requirement: quick-fix button は supported safe-fix rules にのみ表示されなければならない

システムは、局所的かつ安全に自動修正できる markdownlint rule に対してのみ quick-fix button を popup 内へ表示しなければならない（SHALL / MUST NOT）。

#### Scenario: safe fix provider がある rule では fix button を表示する

- **WHEN** diagnostic popup の対象 rule に safe fix provider が登録されている
- **THEN** popup は fix button を表示する
- **THEN** user が fix を実行すると system は対象 diagnostic の範囲に限定して修正を適用する

#### Scenario: safe fix provider がない rule では fix button を表示しない

- **WHEN** diagnostic popup の対象 rule に safe fix provider が存在しない
- **THEN** popup は explanation のみを表示し、実行不能な fix button を表示しない
