## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.19.0 の変更 ID とスコープが確認されていること
- [ ] 既存の Diagnostics 実装、問題検知パネル (Problems Panel)、アーカイブ済みの `markdown-diagnostics` ブランチ/チェンジを再確認していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Supported Rule Catalog and Parity Contract (サポートルール一覧と公式同等性方針)

- [x] 1.1 既存の内部用 diagnostics（問題診断）ルールを一覧にまとめ棚卸しを行い、公式の markdownlint 指定ルールに対応付ける
- [x] 1.2 ユーザーに表示される diagnostics 情報から、内部独自のルール名を排除する（隠蔽する）方針を定義する
- [x] 1.3 対象となるルールが公式の「Rule Code、Title、English description (英語説明)、ドキュメントURL」を持つ「カタログ」データとして管理できるようにする
- [x] 1.4 公式と同等の水準を満たしていない (parity 未達の) 独自ルールの扱いについて、「hidden (隠し扱い) または experimental (実験的扱い)」とするという定義を明確にする

### Definition of Done (DoD)

- [x] ユーザー向けの diagnostics 情報に、内部の独自ルール名が一切表示されないようになっていること
- [x] 同等水準 (parity) に達していないルールの扱いが、プログラム上でもドキュメント上でも明確に分離されていること
- [x] 公式のメタデータ情報の源 (source of truth) が 1 箇所に集約されてブレがないこと
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. Diagnostics Engine Parity (診断エンジンの公式同等化)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 2.1 ユーザー向けに提示する全公式ルールについて、その検知の振る舞い（Behavior）を公式通りに実装、またはそれに合わせて修正する
- [x] 2.2 検知結果 (diagnostics payload) 内に、公式ルールのメタデータ情報を付与する
- [x] 2.3 違反ケースと合格ケースのファイルデータ (fixture) を用いて、挙動に差異がないか検証する回帰テスト (parity regression test) を追加する
- [x] 2.4 既知の開発パターンにおいて、誤検知 (false positive) や検知漏れ (false negative) が十分に抑えられていることを確認する

### Definition of Done (DoD)

- [x] ユーザー提供ルールについて、公式ルールと完全に整合性のある検知結果が返されること
- [x] 将来の実装である「自動修復 (autofix)」を見据え、その際に再利用可能なフォーマット情報 (payload shape) が固定されていること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. Problems Panel UX (問題検知パネルのUX向上)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 3.1 問題検知パネル (Problems Panel) に「公式のルールコード、英語の内容説明、問題の発生箇所 (location)、重大度 (severity)」を表示する
- [x] 3.2 個々のの診断エラー行（item）をクリックした際、エディターおよびプレビューの該当箇所へ一瞬でジャンプできる機能を確認する
- [x] 3.3 公式ドキュメントへのリンク、またはそれと同等の公式参照への導線を追加する
- [x] 3.4 公式水準に達していない (parity 未達の) ルールが、ユーザーに対して公式と同列のものであるかのように誤認されないよう、UI の表示表現で区別されているか確認する
- [x] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] 問題検知パネルだけで、公式から提供されたどのルールの問題なのかの把握から該当箇所へのジャンプまでスムーズに行えること
- [x] 公式の同等性を満たしているルールと、満たしていないルールが UI 上で容易に区別できるよう混同が防がれていること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Refresh and Compatibility Hardening (動作の安定化と互換性の強化)

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 4.1 ユーザー操作による手動更新 (manual refresh) と、保存操作トリガーの自動更新 (save-triggered) の両方において、診断結果パネルが正確に更新される動作を担保する
- [x] 4.2 公式同等ルールと実験的 (experimental) ルールの判定境界に関する回帰テストを追加する
- [x] 4.3 関連するドキュメント (docs) とステータス表示用テキスト (copy) を、v0.19.0 の仕様定義に合わせて最新化する

### Definition of Done (DoD)

- [x] Diagnostics の更新タイミング (refresh policy) が常に一定かつ期待通り (deterministic) に動作すること
- [x] 公式メタデータの内容変化や乖離 (drift) を検出できるテスト、あるいは明確な検証手段が存在していること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 5. Final Verification & Release Work (最終確認とリリース対応)

### Definition of Ready (DoR)

- [x] 1つ前のタスク（Task 4）が完了しており、その成果物（コード・テストを含む）がマージされていること。
- [x] 今回のリリースに関する仕様定義と実装がすべて完了状態にあり、残されたタスクが存在しないこと。

- [x] 5.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [x] 5.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [x] 5.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [x] 5.4 `master` に向けて PR（プルリクエスト）を作成する
- [x] 5.5 `master` へマージする (※ `--admin` の利用は許容される)
- [x] 5.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.19.0` のリリースタグ打ちとリリース作成を実行する
- [x] 5.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする

### Definition of Done (DoD)

- [x] すべてのリリースタスクが正常に完了し、`master` ブランチへのリリース反映コミットがプッシュされていること
- [x] Katana Desktop の稼働環境（GitHub Releases 等）にて `v0.19.0` のリリースオブジェクトが正しく公開されていること
- [x] この OpenSpec change ディレクトリ群（`openspec/changes/v0-19-0-...`）が `/archive` に移動され、ワークスペースからクリーンアップされていること。
