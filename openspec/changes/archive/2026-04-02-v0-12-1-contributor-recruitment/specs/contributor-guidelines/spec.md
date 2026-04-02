## ADDED Requirements

### Requirement: GitHub repository overview から contributor guide を発見できる

システムは、GitHub がリポジトリの `Contributing` 導線を表示できるように、リポジトリルートに正本英語ガイド `CONTRIBUTING.md` を配置しなければならない（SHALL）。

#### Scenario: Repository overview に Contributing タブが表示される

- **WHEN** contributor guide 公開後に、ユーザーが GitHub リポジトリのメインページを開いた時
- **THEN** repository overview に `Contributing` タブが表示される

#### Scenario: Repository sidebar に contributor guide への導線が表示される

- **WHEN** ユーザーが GitHub repository overview を表示した時
- **THEN** repository sidebar に、GitHub が表示した contributor guide への link が含まれる

### Requirement: Contributor guide は日本語版への導線を持つ

システムは、日本語対訳版 guide と、英語版・日本語版 contributor guide 間の明示的な相互リンクを提供しなければならない（SHALL）。

#### Scenario: English contributor guide から日本語版へ移動できる

- **WHEN** 読者が `CONTRIBUTING.md` を開いた時
- **THEN** その文書には `CONTRIBUTING.ja.md` への明確な link が含まれる

#### Scenario: Japanese contributor guide から英語版へ移動できる

- **WHEN** 読者が `CONTRIBUTING.ja.md` を開いた時
- **THEN** その文書には `CONTRIBUTING.md` への明確な link が含まれる

### Requirement: 募集したい contributor 像を明示する

システムは、この時点で KatanA が特に募集中の contributor 像を明確に記載しなければならない（SHALL）。

#### Scenario: Design collaboration を募集する

- **WHEN** 読者が contributor guide を確認した時
- **THEN** guide には、現在の app UI がまだ十分に洗練されていないことを踏まえ、デザイン助言やデザインカンプ提供ができる人を明示的に歓迎する内容が含まれる

#### Scenario: Product and feature direction collaboration を募集する

- **WHEN** 読者が contributor guide を確認した時
- **THEN** guide には、KatanA に何を加え、何を捨て、何を優先するかを一緒に考えられる人を明示的に歓迎する内容が含まれる

#### Scenario: AI-agent-assisted development collaboration を募集する

- **WHEN** 読者が contributor guide を確認した時
- **THEN** guide には、AI エージェントを積極活用する開発 workflow に強い contributor を明示的に歓迎する内容が含まれる

### Requirement: Contributor guide は参加経路を明示する

システムは、guide を読んだ prospective contributor が次にどの参加経路を取るべきかを判断できるようにしなければならない（SHALL）。

#### Scenario: Discussion or issue の入口が分かる

- **WHEN** prospective contributor が、アイデア、design feedback、product direction の意見を共有したい時
- **THEN** guide は、適切な GitHub issue または discussion の導線を示す

#### Scenario: Code contribution の入口が分かる

- **WHEN** prospective contributor が実装作業を pull request として提出したい時
- **THEN** guide は、pull request を開く前に必要な development guide と coding rules 文書へ link する
