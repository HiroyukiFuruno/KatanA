## MODIFIED Requirements

### Requirement: `docs` およびルート `README.md` における2言語サポート体制 (Dual-language support)

`docs/` ディレクトリに含まれる全ドキュメント、およびプロジェクトルートの公開エントリードキュメント (`README.md`, `CONTRIBUTING.md`) は、メインである英語版と並行して、国内ユーザー向けの日本語版を必ず維持管理しなければならない（MUST）。また、日本語ドキュメントの命名は必ず元のファイル名に `.ja` を付与した形式（例: `README.ja.md`, `CONTRIBUTING.ja.md`, `*.ja.md`）に限定しなければならない（MUST）。

#### Scenario: Updating existing `docs/` content

- **WHEN** 作成者が `docs/` 配下の既存ドキュメントを更新した
- **THEN** 作成者は対応する日本語版または英語版も同期して更新しなければならない

#### Scenario: Updating root `README.md`

- **WHEN** 作成者がプロジェクトルートの `README.md` に更新を加えた
- **THEN** 作成者は `README.ja.md` の内容も同期して更新しなければならない

#### Scenario: Updating root `CONTRIBUTING.md`

- **WHEN** 作成者がプロジェクトルートの `CONTRIBUTING.md` に更新を加えた
- **THEN** 作成者は `CONTRIBUTING.ja.md` の内容も同期して更新しなければならない
