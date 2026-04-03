## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.18.0 の変更 ID とスコープが確認されていること
- [ ] 現行の macOS 固定箇所（theme / locale / menu / update / packaging / docs）を `crates/katana-platform`、`crates/katana-ui`、`crates/katana-core`、`scripts`、`.github/workflows` で再確認していること

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Platform Contract と target build の整理

- [ ] 1.1 `katana-platform` に current platform、primary modifier、native menu support、update install mode を扱う platform contract を追加する
- [ ] 1.2 `crates/katana-platform/src/os_theme.rs` と locale 検出経路を Windows / Linux 対応へ拡張し、`crates/katana-ui/src/main.rs` の初回 language 適用まで含めて取得不能時 fallback を明示する
- [ ] 1.3 `crates/katana-platform/build.rs` と `crates/katana-ui/build.rs` の macOS FFI build 条件を整理し、Windows / Linux target で不要な link を引かないことを確認する
- [ ] 1.4 `crates/katana-ui/src/main.rs` の初期 theme / language 適用を platform contract 経由へ寄せる
- [ ] 1.5 `cargo check --target x86_64-pc-windows-msvc` と `cargo check --target x86_64-unknown-linux-gnu` が通る状態を作る
- [ ] 1.6 `settings/defaults.rs`、`settings/service.rs`、locale/theme 検出 helper に対する unit test を追加し、初回起動 fallback と既存ユーザー維持を固定する

### Definition of Done (DoD)

- [ ] macOS / Windows / Linux の platform contract が 1 箇所に整理されていること
- [ ] Windows / Linux target で macOS FFI の link error が発生しないこと
- [ ] 初回起動の theme / language default が platform contract に従うこと
- [ ] locale fallback と既存ユーザー設定維持の回帰テストが追加されていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. Command Surface と shortcut のクロスプラットフォーム化

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 macOS では既存 native menu を維持し、Windows / Linux では `AppAction` 群へ到達できる in-app command surface を追加する
- [ ] 2.2 `OpenWorkspace`、`SaveDocument`、`ToggleSettings`、`CheckForUpdates`、`ShowReleaseNotes`、language change を各 OS で同等に到達可能にする
- [ ] 2.3 `Cmd` 固定 shortcut を primary modifier abstraction へ置き換え、Windows / Linux では `Ctrl` として動作させる
- [ ] 2.4 `crates/katana-ui/src/native_menu.rs`、`crates/katana-ui/src/shell_ui.rs`、必要なら top bar UI を更新し、non-macOS の command surface と shortcut が既存 preview / workspace 操作を壊さないことを確認する
- [ ] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] macOS は native menu、Windows / Linux は in-app command surface で同等の主要 command に到達できること
- [ ] 検索などの primary shortcut が macOS では `Command`、Windows / Linux では `Ctrl` で動作すること
- [ ] 既存の workspace / preview の UI 導線が回帰していないこと
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Font / Emoji / Branding の runtime 品質保証

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 3.1 `crates/katana-platform/src/os_fonts.rs` を cross-platform の font directory 探索へ拡張する
- [ ] 3.2 `crates/katana-ui/src/font_loader/*` と `katana_core::markdown::color_preset` の candidate chain を見直し、Windows / Linux でも editor / preview が readable に表示されるようにする
- [ ] 3.3 emoji font candidate が利用できない場合でも crash せず fallback する経路を追加または明示する
- [ ] 3.4 icon / splash / window icon の表示が Windows / Linux でも識別可能であることを確認する
- [ ] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### 完了条件 (DoD)

- [ ] Windows / Linux で font 探索 failure が startup crash を起こさないこと
- [ ] default font fallback により editor / preview が readable に表示されること
- [ ] emoji font 不在時も recoverable fallback で描画を継続すること
- [ ] 対応 OS でアプリケーションアイコンが識別可能に表示されること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Update Policy と release artifact の platform-aware 化

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 4.1 `crates/katana-core/src/update/version.rs` の asset 解決を platform / architecture aware に変更する
- [ ] 4.2 `crates/katana-core/src/update/installer.rs` と update UI を見直し、macOS は auto-install、Windows / Linux は manual download に切り替える
- [ ] 4.3 `Makefile`、`scripts/package-mac.sh`、`scripts/release/*`、`.github/workflows/release.yml` を整理し、`KatanA-windows-x86_64.zip` と `KatanA-linux-x86_64.tar.gz` を build / publish できるようにする
- [ ] 4.4 `.github/workflows/ci.yml` と `.github/workflows/release.yml` を Windows / Ubuntu を含む matrix へ更新する
- [ ] 4.5 Windows / Ubuntu CI job で build / smoke verification の log と生成 artifact を macOS からレビューできる形で保持する
- [ ] 4.6 update dialog と release artifact 導線が platform policy に従い、Windows / Linux で auto-install を示す文言が残らないことを確認する
- [ ] 4.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 4.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] macOS / Windows / Linux で matching release asset 名が決定できること
- [ ] Windows / Linux で update action が broken install path を実行しないこと
- [ ] release workflow が macOS `.dmg` / `.zip`、Windows `.zip`、Linux `.tar.gz` を生成できること
- [ ] CI に Windows / Ubuntu の build または smoke verification と review 用の log / artifact retention が追加されていること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 5. Docs / Support Matrix / Verification

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 5.1 `README.md` / `README.ja.md` の platform badge、support matrix、install 手順、update 説明を Windows / Linux を含む内容へ更新する
- [ ] 5.2 `docs/development-guide.md` / `docs/development-guide.ja.md` の prerequisites、build 手順、supported OS 記述を更新する
- [ ] 5.3 macOS を主開発環境とする maintainer 向けに、Windows / Linux support の検証レーン（参照すべき CI job / artifact / log / manual verification の入口）を文書化する
- [ ] 5.4 Windows / Linux それぞれについて、VM / remote machine / physical machine のいずれでも実行できる runtime smoke checklist を作成し、startup、workspace open、Markdown 編集、preview 表示の required evidence を定義する
- [ ] 5.5 release blocking 条件として、Windows / Ubuntu CI 成功、artifact 生成、manual verification evidence 確認の判定順を文書化する
- [ ] 5.6 OpenSpec の spec / design / tasks と実装対象 file の対応関係が崩れていないことを確認する

### Definition of Done (DoD)

- [ ] repository root の公開文書から macOS only 表現が除去されていること
- [ ] Windows / Linux 向け install / build / update / verification の説明が読者に分かること
- [ ] Windows / Linux の runtime smoke checklist と required evidence が文書化されていること
- [ ] support matrix、artifact 名、update policy、verification lane の説明が proposal / design / specs と整合していること
- [ ] `make check` が exit code 0 で通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 6. 最終確認とリリース作業

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 6.2 Ensure `make check` passes with exit code 0
- [ ] 6.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 6.4 Create a PR targeting `master`
- [ ] 6.5 Merge into master (※ `--admin` is permitted)
- [ ] 6.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.18.0`
- [ ] 6.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
