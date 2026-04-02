## Context

KatanA には既に `README.md` / `README.ja.md` と `docs/` 配下の英日ドキュメントがあるが、GitHub repository overview から直接見つけられる contributor guide はまだ存在しない。
現状の contributing 情報は `docs/development-guide.md` 内に埋まっており、募集したい contributor 像も一般論に留まっている。

今回は単に「OSS なので誰でも歓迎」と書くのではなく、現時点の KatanA が特に求めている協力者を明示する。

- UI/UX の洗練に助言やデザインカンプをくれる人
- 今後 KatanA に何を入れ、何を捨て、何を優先するかを一緒に考えられる人
- AI エージェントを積極活用した開発に強い人

さらに、GitHub Docs の contributor guidelines 仕様に沿って repository overview から discoverable にし、英語版と日本語版を二重管理する必要がある。

## Goals / Non-Goals

**Goals:**

- GitHub repository overview に `Contributing` タブが表示される contributor guide を提供する
- 英語版 `CONTRIBUTING.md` を正本として、日本語版 `CONTRIBUTING.ja.md` を同期管理する
- 募集したい contributor profile を具体的に明示する
- README から contributor guide へ辿れるようにする
- 他の AI エージェントが会話コンテキストなしで実装できるよう、doc placement と sync rule を明確化する

**Non-Goals:**

- contributor onboarding フロー全体の自動化
- issue template や PR template の全面再設計
- デザインカンプ自体の制作
- project governance や maintainer 権限ポリシーの策定

## Decisions

### 1. Canonical contributor guide はルート `CONTRIBUTING.md` に置く

GitHub Docs 上、contributing guidelines は repository root、`docs/`、`.github/` のいずれでも認識される。ただし今回は英日 2 ファイルを公開エントリードキュメントとして管理したいため、ルートに `CONTRIBUTING.md` と `CONTRIBUTING.ja.md` を置く。

- 採用理由:
  - GitHub repository overview 上の `Contributing` タブ要件を満たしやすい
  - `README.md` / `README.ja.md` と同じ bilingual public-entry pattern に揃う
  - `.github/` 優先順位や community health file precedence の曖昧さを避けられる
- 代替案:
  - `.github/CONTRIBUTING.md`: GitHub 側では有効だが、JA 版との並びと public-entry docs の運用が見えにくい
  - `docs/CONTRIBUTING.md`: overview surface は可能でも、入口として一段深くなるため不採用

### 2. 英語版を GitHub surface 用の正本とし、日本語版は対訳 companion にする

GitHub 上で直接 surfacing されるのは `CONTRIBUTING.md` であるため、英語版を canonical とする。日本語版は `CONTRIBUTING.ja.md` とし、両者に相互リンクを置く。

- 採用理由:
  - 既存の English-first public docs 運用と整合する
  - GitHub surface の挙動と bilingual support を両立できる
  - 他の AI エージェントが「どちらを先に直すべきか」を迷いにくい
- 代替案:
  - 1ファイル内 bilingual 併記: 可読性が落ち、差分管理もしづらい
  - 日本語版を正本にする: GitHub 上の第一印象と既存運用に合わないため不採用

### 3. 募集内容は generic contribution guide ではなく targeted recruitment として書く

今回の目的は contributor process の完全網羅ではなく、「いま KatanA がどのような協力者を求めているか」を伝えることである。そのため contributor guide には次の募集軸を明示する。

- design advice / UI feedback / design comps
- feature ideas と product direction の壁打ち
- AI-agent-assisted development に強い implementation contributors

- 採用理由:
  - 現在の project need と募集メッセージが一致する
  - generic 文面より、来てほしい人に届きやすい
  - README の「アイデアを募集しています」より一歩踏み込んだ recruiting ができる
- 代替案:
  - 一般的な OSS 貢献案内だけに留める: 今回の募集意図が弱くなるため不採用

### 4. 既存 README は入口、CONTRIBUTING は詳細、development guide は実装前提に分離する

README は project overview と short CTA に留め、詳細な contributor recruiting と participation guidance は `CONTRIBUTING.md` に集約する。`docs/development-guide.md` は build/test/setup の詳細に集中させる。

- 採用理由:
  - 役割分担が明確になる
  - 他の AI エージェントが public docs の責務を判断しやすい
  - contributor recruiting message と developer setup 情報の混線を避けられる
- 代替案:
  - development guide に recruiting を残す: discoverability が低く、GitHub overview DoD を満たさない

### 5. DoD の中核は GitHub UI 上の discoverability で確認する

今回の成否はファイルを置いたこと自体ではなく、GitHub repository overview から contributor guide が見つかることにある。したがって DoD は local file existence ではなく、GitHub の `Contributing` tab と sidebar link の表示確認を含む。

- 採用理由:
  - user-facing outcome と一致する
  - GitHub Docs の仕様と直接整合する
  - 「正しい場所に置いたつもりだが surface されない」事故を防げる
- 代替案:
  - ファイル存在のみを DoD にする: user outcome を保証しないため不採用

## Risks / Trade-offs

- [Risk] 英語版と日本語版の内容がズレる -> Mitigation: `document-organization` に同期更新 requirement を入れ、相互リンクも必須化する
- [Risk] contributor guide と development guide の内容が重複する -> Mitigation: recruiting / participation guidance と build/test/setup の責務を分離する
- [Risk] GitHub surface 条件を誤解して tab が表示されない -> Mitigation: canonical file name を正確に `CONTRIBUTING.md` とし、repository overview 上で手動確認する
- [Risk] 募集内容が抽象的すぎて来てほしい人に刺さらない -> Mitigation: design, product direction, AI-agent development の 3 軸を明示する
