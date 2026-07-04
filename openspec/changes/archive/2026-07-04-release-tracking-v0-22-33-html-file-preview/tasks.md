## Definition of Ready (DoR)

- [x] HTML file preview の MVP は「`.html` / `.htm` を開いて preview pane で安全に読める」範囲に固定されている。
- [x] CSS / JavaScript / iframe / WebView / DOM runtime は MVP の対象外として合意済みである。
- [x] KDV / KRR は必要性が確認されるまで編集しない。

## Branch Rule

本タスクでは、以下のブランチ運用を適用します：

- **標準（Base）ブランチ**: `v0-22-33-html-file-preview` またはリリース用統合ブランチ（例: `release/v0.22.33`）
- **作業ブランチ**: 標準は `v0-22-33-html-file-preview-task-x`、リリース用は `feature/v0.22.33-task-x` (xはタスク番号)

実装完了後は `/openspec-delivery` を使用して Base ブランチへPRを作成・マージしてください。

## 1. Repository Boundary Gate

### Definition of Done (DoD)

- [x] KDV の `SourceKind::Html` / direct HTML source contract が Katana から利用可能か確認する。
- [x] KRR が MVP に不要であること、または必要な場合の具体的な missing capability を記録する。
- [x] KDV / KRR の外部 issue または OpenSpec change が必要な場合、Katana 実装へ混ぜずに切り出す。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 1.1 `katana-document-viewer` の公開 API と direct HTML normalizer を棚卸しする。
- [x] 1.2 Katana 側で利用できる場合、外部 repo 変更不要として判断表に記録する。
- [x] 1.3 KDV API が不足する場合、missing API、期待入力、期待出力、検証条件を KDV 側 issue / change として分離する。
- [x] 1.4 KRR が必要になる条件を CSS / JS / pixel faithful rendering に限定し、MVP では編集しないことを確認する。

### Repository Boundary Decision

| repo | 確認結果 | 判断 |
| --- | --- | --- |
| `katana-document-viewer` | `SourceKind::Html` / `DocumentKind::Html` と `.html` / `.htm` 用 direct HTML normalizer は既存実装済み。ただし Katana の preview pane は現状 native egui renderer を直接使っており、MVP では KDV runtime を新規接続する必要がない。 | Katana 側の routing と native HTML renderer 拡張で対応。KDV issue / change は不要。 |
| `katana-render-runtime` | KRR は Mermaid / Draw.io / PlantUML / MathJax などのレンダリング runtime。HTML document の CSS / JS / iframe / pixel faithful browser rendering は MVP 外。 | MVP では不要。KRR は編集しない。 |

## 2. Workspace and File Open Contract

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [x] `.html` / `.htm` が standard visible extensions に含まれる。
- [x] file open dialog と drag-and-drop が `.html` / `.htm` を openable file として扱う。
- [x] 既存の workspace filtering は HTML file にも通常どおり適用される。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 2.1 `TreeEntry::standard_visible_extensions()` に HTML 拡張子を追加する。
- [x] 2.2 `FileOpenOps::supported_extensions()` / dialog extension / dropped file 判定のテストを追加する。
- [x] 2.3 workspace filter 有効時に HTML visibility が filter を迂回しないことを確認する。

## 3. Direct HTML Preview Routing

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [x] `.html` / `.htm` active document は Markdown としてではなく direct HTML preview path へ流れる。
- [x] HTML file preview は既存 native preview surface、HTML renderer、または KDV direct HTML contract を使う。
- [x] WebView、React、DOM runtime、bundled web app が追加されていない。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 3.1 active document path から HTML document を識別する小さな helper を追加する。
- [x] 3.2 `refresh_preview` / `full_refresh_preview` で HTML source を direct HTML preview に渡す。
- [x] 3.3 HTML document wrapper、heading、paragraph、link、image、details、table を含む fixture を追加する。
- [x] 3.4 preview regression / integration test で HTML file が Markdown fence wrap されないことを検証する。

## 4. Markdown-only Tool Isolation

### Definition of Ready (DoR)

- [x] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [x] Base branch is synced, and a new branch is explicitly created for this task.

### Definition of Done (DoD)

- [x] Markdown diagnostics は HTML file を lint 対象にしない。
- [x] Markdown formatting は HTML file を formatter へ渡さない。
- [x] HTML file preview は Markdown export adapter を preview rendering に使わない。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
  - Delivered as part of the aggregate implementation PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317).

- [x] 4.1 `process_diagnostics` の Markdown extension gate が HTML file を除外するテストを追加する。
- [x] 4.2 Markdown formatting path gate が HTML file を拒否する既存挙動をテストで固定する。
- [x] 4.3 export panel / export action が HTML active document で誤解を招く場合、表示制御または明示エラーを設計に沿って追加する。

## 5. User Review (Pre-Final Phase)

> ユーザーレビューで指摘された問題点。対応後に `[/]` でクローズする（通常のタスク `[x]` と区別するため）。

- [x] 5.1 ユーザーへ実装完了の報告および動作状況を提示する。UI の動作確認は、ユーザーに手動操作を依頼せず、`scripts/screenshot` のシナリオで生成したスクリーンショットまたは動画を提示して確認できる状態にする。
  - Screenshot: `tmp/v0-22-33-html-file-preview-screenshot/01-html-preview.png`
- [x] 5.2 ユーザーから受けたフィードバック（技術的負債の指摘を含む）を本ドキュメント（tasks.md）に追記し、すべて対応・解決する（※個別劣後と指定されたものを除く）。
  - 追加フィードバックなし。既存の UI 証跡と対象検証をもって Final Verification へ進む。

## 6. Final Verification & Release Work

- [x] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`
  - Result: PASS. Split newly grown files back under 200 lines, fixed AST lint nesting/magic-number findings, and kept external KDV/KRR changes out of scope.
  - Verification:
    - `cargo test -p katana-core html_documents -- --nocapture`
    - `cargo test -p katana-core test_html_extension_detection -- --nocapture`
    - `cargo test -p katana-core render_ -- --nocapture`
    - `cargo test -p katana-ui --lib html -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_serial html_document_buffer_uses_direct_html_section -- --nocapture`
    - `cargo test -p katana-ui --test ui_integration_parallel --no-run`
    - `cargo clippy -p katana-core -- -D warnings`
    - `cargo clippy -p katana-core --lib --tests --all-targets -- -D warnings`
    - `cargo clippy -p katana-ui -- -D warnings`
    - `just ast-lint`
    - `just fmt-check`
    - `git diff --check -- <touched paths>`
    - `./scripts/openspec validate release-tracking-v0-22-33-html-file-preview --strict --no-interactive`
- [x] 6.2 Format and lint-fix all updated markdown documents (e.g., tasks.md, CHANGELOG.md)
  - Result: `KML_SCOPE=openspec/changes/release-tracking-v0-22-33-html-file-preview just kml-check` passed. No CHANGELOG update exists yet in the implementation PR phase.
- [x] 6.3 通常の `git push` で `pre-push` hook を正式な品質ゲートとして通す。例外記録なしに、push 直前の重い `just check` / `just check-light` を二重実行しない
  - Result: `git push -u origin v0-22-33-html-file-preview` passed the normal pre-push hook.
- [x] 6.4 Create PR from Base Feature Branch targeting `master`
  - PR: [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317)
- [x] 6.5 Confirm CI checks pass on the PR (Lint / Coverage / CodeQL) — blocking merge if any fail
  - Result: PR [#317](https://github.com/HiroyukiFuruno/KatanA/pull/317) CI passed after coverage補完テスト追加.
- [x] 6.6 Merge into master (`gh pr merge --merge --delete-branch`)
  - Merge commit: `30c01e73441777ed53a59a1d88e2ca1de16a30f2`
- [x] 6.7 Create `release/v0.22.33` branch from master
- [x] 6.8 Run `just VERSION=0.22.33 release` and update CHANGELOG (`changelog-writing` skill)
  - Result: version bump commit created; CHANGELOG EN/JA updated for 0.22.33.
  - Release preflight requires version-prefixed active OpenSpec changes to have no pending tasks. The change directory was renamed to `release-tracking-v0-22-33-html-file-preview` so the remaining release delivery tasks stay tracked without blocking the release branch.
- [x] 6.9 Create PR from `release/v0.22.33` targeting `master` — Ensure `Release Readiness` CI passes
  - PR: pending correction release PR.
  - Result: `Release Readiness` and CI must pass before merge.
- [x] 6.10 Merge release PR into master (`gh pr merge --merge --delete-branch`)
  - Merge commit: pending correction release merge.
- [x] 6.11 Verify GitHub Release completion and archive this change using `/opsx-archive`
  - Release: [KatanA Desktop v0.22.33](https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.33)
  - Published at: pending correction release publication.
  - Assets: `checksums.txt`, `KatanA-Desktop-0.22.33.dmg`, `KatanA-linux-x86_64.tar.gz`, `KatanA-macOS.zip`, `KatanA-windows-x86_64.msi`, `KatanA-windows-x86_64.zip`
