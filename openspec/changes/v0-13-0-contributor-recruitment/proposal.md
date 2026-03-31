## Why

KatanA はまだ UI とプロダクトの方向性を磨いている段階であり、いま必要なのは単なる一般的な OSS 貢献案内ではなく、どのような人に来てほしいかを明確に示す公開ドキュメントです。
特に design feedback、design comps、機能や方向性の議論相手、そして AI エージェント活用開発に強い協力者を、GitHub の repository overview から見つけやすい形で募集する必要があります。

## What Changes

- リポジトリ直下に英語版 `CONTRIBUTING.md` と日本語版 `CONTRIBUTING.ja.md` を追加し、二重管理する
- 英語版を GitHub 上で discoverable な正本とし、日本語版への導線を明示する
- 募集内容として、design advice / design comps、feature and product direction collaboration、AI-agent-assisted development collaboration を明示する
- `README.md` と `README.ja.md` から contributor guide へ導線を追加する
- GitHub repository overview に `Contributing` タブが表示される状態を DoD として定義する

## Capabilities

### New Capabilities

- `contributor-guidelines`: 公開 contributor guide を通じて、募集したい協力者像、参加経路、GitHub 上での discoverability を定義する

### Modified Capabilities

- `document-organization`: ルート公開ドキュメントの二言語管理対象に `CONTRIBUTING.md` を含める

## Impact

- ルート公開ドキュメント: `CONTRIBUTING.md`, `CONTRIBUTING.ja.md`, `README.md`, `README.ja.md`
- 既存の開発者向け導線: `docs/development-guide.md`, `docs/development-guide.ja.md` への参照整理が必要なら調整
- GitHub repository overview: `Contributing` タブと sidebar link の表示確認
- OpenSpec: contributor recruiting と bilingual public docs 管理の要件追加
