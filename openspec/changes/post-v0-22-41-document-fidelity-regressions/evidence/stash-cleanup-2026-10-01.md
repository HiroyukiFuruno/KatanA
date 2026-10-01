# stash整理と作業ブランチ同期（2026-10-01）

ユーザー依頼により、共通stash3件を復元可能なGit bundleへ保存し、stash一覧を整理する。正式履歴への統合完了とは扱わない。

## 保全

- bundle: `tmp/stash-archive/2026-10-01/stashes.bundle`（このrelease worktree内）
- SHA-256: `14255f671791452d492e2ad287c064512e5f34462d01b8ea44ddb6f145507e7a`
- `git bundle verify`: 成功。3件のstashコミット、親コミット、未追跡ファイルのtreeを含む完全な履歴。
- index 0: `8b60f27e81ad0d349d7c3a616a0efae1fabd28e0`（document-fidelity）
- index 1: `670f8dd436120a7e6bea9b093beafa59fbfba761`（KLE test recovery）
- index 2: `1d32b220e0f5a5b6c4c5f4bc07df93ea4e61a8b5`（KLE adapter）

## 照合結果と残DoD

- document-fidelity: 36ファイル中14ファイルは作業ツリーと完全一致。22ファイルは現版との差分があり、旧版を上書きしていない。
- KLE recovery: 12ファイル中11ファイルは現版との差分あり。`crates/katana-ui/tests/integration/editor/kle_downstream_adapter.rs`は現作業ツリーに存在しない。
- KLE adapter: 9ファイル中8ファイルは現版との差分あり。同テストファイルは現作業ツリーに存在しない。
- [x] 異なる現版の変更意図とKLEテストを選別し、必要な変更を現作業ツリーへ取り込み、不採用根拠を確定した。下記に検証結果を記録する。
- [ ] 採用差分を既存リリース差分とともに検証・commitし、正式履歴へ統合する。bundle化や現作業ツリーへの適用だけを正式統合完了としない。
- 復元時は `git fetch <bundle絶対パス> refs/stash-archive/2026-10-01/<index>` で取得し、`git show FETCH_HEAD`で内容を確認できる。無条件のstash applyは行わない。

## 上流同期

- masterは`0561692f`でorigin/masterと一致し、変更・未追跡ファイルなし。
- release/v0.22.42のみ1コミット遅れていた。`#321`のGitHub Actions依存更新をfast-forwardで取り込んだ。
- 競合する作業中workflowは`tmp/stash-archive/2026-10-01/build-and-release.before-sync.yml`に保全した後、作業差分を復元し、上流のAction SHA/版更新を新規smoke jobにも反映。
- `git diff --check`成功。release HEADとorigin/masterのahead/behindは`0/0`。リモートブランチは削除していない。

## KLE旧差分の採否と検証

- 採用: `integration/editor/navigation.rs`のExplorer／複数タブ回帰2件を現APIへ移行し、実行対象`ui_integration_parallel.rs`へ登録した。
- `cargo test --locked -p katana-ui --test ui_integration_parallel editor_navigation:: -- --nocapture`: 2件成功、143件はfilter対象外。全体テスト成功とは扱わない。
- `cargo clippy --locked -p katana-ui --test ui_integration_parallel -- -D warnings`: 成功。
- 不採用: KLEのpath開発依存と、それを支えるCIのsibling clone／mount。公開registry依存のみを使用する今回のリリース制約に反する。
- 不採用: 旧`kle_downstream_adapter.rs`とfixture map／test_hooks一式。人工的なMD999診断、fix変換、画像fixtureを注入する構成であり、実ランタイムの受入証拠にならない。
- 不採用: context menuの旧3テスト。現実装でラベル前提、整形結果の末尾改行、保存前提が一致せず失敗した。テストを通すために商用コードを変更せず、旧内部状態・固定待機に依存するscroll/selectの検証も移植しない。
- 正式commitとリリース全gateは未完了。現作業ツリーへの採用と、正式履歴・公開を区別する。

## document-fidelity旧差分の内容照合表

36ファイルを現作業ツリーと照合した。14ファイルは完全一致、22ファイルは機能分離・診断共通化・後続修正・進捗更新による差分であり、旧版の無条件再適用は後続修正を戻してしまう。下記はソース内容の照合であり、全gateや正式commitの完了証跡ではない。

| 旧stashの対象 | 判定・現版での根拠 |
|---|---|
| `crates/katana-ui/src/app/action/dispatch.rs` | dispatch_panels.rsへ分離。PreviewPanel型で同じ4メニューの可否判定・閉鎖・排他制御を維持。 |
| `crates/katana-ui/src/app/action/process_helpers.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/font_loader/helpers.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/font_loader/mod.rs` | 旧サイズ上限を保持し、owned_bytes診断を追加。 |
| `crates/katana-ui/src/font_loader/tests/definitions.rs` | 旧巨大emoji除外検証を保持し、所有フォント量の検証を追加。 |
| `crates/katana-ui/src/font_loader/tests/emoji.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/preview_pane/document_surface/controls_sheet_tabs.rs` | 旧下端タブ予約・実hit target回帰を保持。railを44pxへ拡張して水平scrollbarとの重なりを解消し、ラベルロジックを分離。 |
| `crates/katana-ui/src/preview_pane/document_surface/mod.rs` | 診断・タブ等の旧module導線を保持し、現版の追加moduleを維持。 |
| `crates/katana-ui/src/preview_pane/document_surface/painter_page.rs` | texture診断をDebugLog::writeへ統一。旧計測値・イベントを保持。 |
| `crates/katana-ui/src/preview_pane/document_surface/render.rs` | worker handoff診断・下端タブ予約を保持。現版レイアウト/resource制御の拡張を維持。 |
| `crates/katana-ui/src/preview_pane/document_surface/render_events.rs` | frame受信診断をDebugLog::writeへ統一。旧計測値を保持。 |
| `crates/katana-ui/src/preview_pane/document_surface/source.rs` | local/remote intake診断・byte_lenを保持。DebugLog::writeへ統一。 |
| `crates/katana-ui/src/preview_pane/document_surface/tests.rs` | 実Office入力回帰を保持。featureで実受入を分離し、誤ったdebug worker選択をrelease worker明示指定へ修正。 |
| `crates/katana-ui/src/preview_pane/document_surface/types.rs` | started_at・drop診断・command channel closeを保持。DebugLog::writeへ統一。 |
| `crates/katana-ui/src/preview_pane/document_surface/worker.rs` | session open/close診断とclose処理を保持。DebugLog::writeへ統一。 |
| `crates/katana-ui/src/preview_pane/document_surface/worker_tests.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/preview_pane/image_html_surface.rs` | 1280x900/#s15/sticky pixel比率の旧回帰を保持。実入力用feature構成を維持。 |
| `crates/katana-ui/src/preview_pane/image_html_surface_pane.rs` | 旧wait_for_frame_for_testの公開範囲拡張を保持し、ホストの追加観測APIを維持。 |
| `crates/katana-ui/src/shell/shell_tests.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/state/command_inventory/view_commands.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/theme_bridge/logic.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/views/panels/preview/mod.rs` | 旧PreviewMenu/PreviewMenuAvailabilityの再exportを保持。分離したsidebar moduleを追加。 |
| `crates/katana-ui/src/views/panels/preview/side_panels.rs` | 旧可否判定・強制closeを保持。実ボタン非活性/hover制御はsidebar系moduleへ分離。 |
| `crates/katana-ui/src/views/panels/preview/toc_availability.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/tests/integration/preview_pane/mod.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `crates/katana-ui/src/preview_pane/document_surface/debug_log.rs` | 旧DEBUG=true限定関数をcrate::debug_log::DebugLogへ統一。exact true判定回帰を保持。 |
| `crates/katana-ui/tests/integration/preview_pane/menu_availability.rs` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/.openspec.yaml` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/design.md` | 旧要件・制約を保持。リスク記述のmarkdown整形と現版診断/受入契約の追記。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/baseline-memory.md` | 旧baselineを保持。追加実装後の計測を記録し旧『gate pending』を更新。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/evidence/supplied-office-first-frame.md` | 旧KDV0.5.5/debug worker計測は現リリースworker結果で更新。実アプリ受入未完了の区別は保持。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/proposal.md` | 旧責務境界を保持。Markdown画像parser指摘を追加。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/specs/document-preview-resource-lifecycle/spec.md` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/specs/html-file-preview/spec.md` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/specs/multi-format-document-preview/spec.md` | 現作業ツリーとbyte単位で完全一致。再適用不要。 |
| `openspec/changes/post-v0-22-41-document-fidelity-regressions/tasks.md` | 旧DoDを保持して拡張。旧進捗の上書き復元は行わない。 |
