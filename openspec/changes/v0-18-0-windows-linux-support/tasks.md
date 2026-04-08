## Definition of Ready (DoR)

- [ ] `proposal.md`、`design.md`、`specs` が揃っていること
- [ ] 対象バージョン 0.18.0 の変更 ID とスコープが確認されていること
- [ ] `crates/katana-platform`, `crates/katana-ui`, `crates/katana-core`, `scripts`, `.github/workflows` 内に存在する現行の macOS 限定仕様 (テーマ / ロケール / メニュー / アップデート / パッケージング / ドキュメント等) を再確認していること

## Branch Rule

Tasks Grouped by ## = 各実装セッション中は、`/openspec-branching` ワークフロー (`.agents/workflows/openspec-branching.md`) で定義されたブランチ運用基準に無条件で従うこと。

---

## 1. Platform Contract とターゲットビルド環境の整理

- [ ] 1.1 `katana-platform` に、現在の OS、主要修飾キー (primary modifier)、ネイティブメニューのサポート有無、アップデートインストール方法を判定する「Platform Contract (プラットフォーム共通契約)」を追加する
- [ ] 1.2 `crates/katana-platform/src/os_theme.rs` およびロケール検出経路を Windows / Linux 対応へ拡張し、`crates/katana-ui/src/main.rs` の初回言語適用時において、取得不能時のフォールバック動作を明示する
- [ ] 1.3 `crates/katana-platform/build.rs` と `crates/katana-ui/build.rs` の macOS 用 FFI ビルド条件を整理し、Windows / Linux をターゲットとした際に不要なリンクエラーが発生しないようにする
- [ ] 1.4 `crates/katana-ui/src/main.rs` での初期テーマおよび言語適用処理を、導入した Platform Contract 経由に寄せる
- [ ] 1.5 `cargo check --target x86_64-pc-windows-msvc` および `cargo check --target x86_64-unknown-linux-gnu` が正常に通る環境を構築する
- [ ] 1.6 `settings/defaults.rs`、`settings/service.rs`、およびロケール/テーマ検出ヘルパーに対する単体テスト (unit test) を追加し、初回起動時のフォールバック動作と既存ユーザーの設定が保持されることを保証する

### Definition of Done (DoD)

- [ ] macOS / Windows / Linux 向けの Platform Contract の定義が 1 箇所へ整理されていること
- [ ] Windows / Linux ターゲットに設定した際、macOS FFI のリンクエラーが発生しないこと
- [ ] 初回起動時のテーマおよび言語のデフォルト設定が Platform Contract に従っていること
- [ ] ロケールフォールバック動作と既存ユーザーの設定維持が、回帰テストで担保されていること
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 2. コマンド実装 (Command Surface) とショートカットのクロスプラットフォーム化

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [ ] 2.1 macOS では既存のネイティブメニューを維持しつつ、Windows / Linux 向けにはすべての `AppAction` 命令へ到達できるアプリ内コマンド領域 (in-app command surface) を追加する
- [ ] 2.2 ワークスペースを開く (OpenWorkspace)、ドキュメントの保存 (SaveDocument)、設定パネルの開閉 (ToggleSettings)、更新確認 (CheckForUpdates)、リリースノート表示 (ShowReleaseNotes)、言語切り替え等の主要操作が全 OS で同等に利用可能にする
- [ ] 2.3 `Cmd` キーに固定されているショートカットを「主要修飾キーの抽象化機能」へ置き換え、Windows / Linux 利用時は自動的に `Ctrl` キーを使用するように動作させる
- [ ] 2.4 `crates/katana-ui/src/native_menu.rs`, `crates/katana-ui/src/shell_ui.rs`、および必要に応じて上部バー UI を更新し、macOS 以外の操作サーフェスやショートカット変更が、既存のプレビュー機能やワークスペース操作を破壊しないことを保証する
- [ ] 2.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [ ] 2.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [ ] macOS はネイティブメニュー、Windows / Linux はアプリ内のコマンドUI から、同等の主要コマンド機能にアクセスできること
- [ ] 検索等の主要なショートカットキーが、macOS では `Command`、Windows / Linux では `Ctrl` として動作すること
- [ ] 既存のワークスペース・プレビュー関連の UI 動線に回帰バグが生じていないこと
- [ ] `make check` がエラーなし (exit code 0) で通過すること
- [ ] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 3. フォント / 絵文字 / ブランディング要素のランタイム品質保証

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 3.1 `crates/katana-platform/src/os_fonts.rs` の処理をクロスプラットフォーム対応のフォントディレクトリ探索へ拡張する
- [x] 3.2 `crates/katana-ui/src/font_loader/*` および `katana_core::markdown::color_preset` の動作を見直し、Windows や Linux でもエディターおよびプレビュー内の文字が明確に可読領域 (readable) として表示されるようにする
- [x] 3.3 絵文字フォントが存在しない、あるいは利用できない環境でもアプリケーションがクラッシュせず、適切にフォールバック表示する経路を追加（または明示）する
- [x] 3.4 アイコン、スプラッシュスクリーン、ウィンドウアイコンなどが Windows / Linux 環境下でも十分に識別可能であることを確認する
- [x] 3.5 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 3.6 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] Windows / Linux にてフォント探索に失敗しても、起動クラッシュが発生しないこと
- [x] デフォルトフォントのフォールバック機能により、エディター・プレビューの双方が十分に読める形で表示されること
- [x] 絵文字フォントが不在の環境でも、クラッシュせずフォールバックによる描画が継続されること
- [x] 各対応 OS の環境で、アプリケーションのアイコンなどが意図通り識別可能に表示されていること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 4. Update Policy とリリース成果物のプラットフォーム対応 (Platform-aware 化)

### Definition of Ready (DoR)

- [ ] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [ ] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 4.1 `crates/katana-core/src/update/version.rs` のアセット解決を、動作プラットフォームおよびアーキテクチャ (OS / CPU) に適応するよう変更する
- [x] 4.2 `crates/katana-core/src/update/installer.rs` およびアップデート通知時の UI を見直し、macOS は自動インストール、Windows / Linux に関しては手動ダウンロードへ切り替える設計とする
- [x] 4.3 `Makefile`、`scripts/package-mac.sh`、`scripts/release/*`、`.github/workflows/release.yml` などのスクリプト類を整理し、`KatanA-windows-x86_64.zip` と `KatanA-linux-x86_64.tar.gz` をビルドおよびパブリッシュできるよう機能拡張する
- [x] 4.4 `.github/workflows/ci.yml` と `.github/workflows/release.yml` に対して、Windows と Ubuntu をビルド実行環境 (matrix) に加える
- [x] 4.5 Windows / Ubuntu の CI ジョブにおける「ビルド結果」および「スモークテストの検証」のログや生成アセット (artifact) を、macOS 側からレビューできる形で保持、収集する
- [x] 4.6 プラットフォームポリシーに従ってアップデートダイアログやリリースへの動線が表示されることを確認し、Windows / Linux には自動インストールを示すような文言が残らないよう徹底する
- [x] 4.7 ユーザーへのUIスナップショット（画像等）の提示および動作報告
- [x] 4.8 ユーザーからのフィードバックに基づくUIの微調整および改善実装

### Definition of Done (DoD)

- [x] macOS / Windows / Linux 向けに適切なリリースアセット名が自動的に判別・決定できるよう作られていること
- [x] Windows / Linux では、アップデート処理時に存在しないインストールパスなどを実行する不具合（broken install path）が起きない想定であること
- [x] リリース用のワークフローが、macOS 向け (`.dmg` / `.zip`)、Windows 向け (`.zip`)、Linux 向け (`.tar.gz`) をすべて生成できること
- [x] GitHub CI 上に、Windows / Ubuntu のビルドおよびスモークテスト、テスト結果やアセットをレビュー用として保持する仕組みが追加されていること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 5. Docs / Support Matrix / Verification の文書化と整備

### Definition of Ready (DoR)

- [x] 1つ前のタスクがデリバリサイクル（自己レビュー、必要に応じたリカバリ、PR作成、マージ、ブランチ削除）を完全に終えていること。
- [x] ベースブランチが最新化（同期）されており、このタスク用に新しいブランチが明示的に作成されていること。

- [x] 5.1 `README.md` および `README.ja.md` に記載されているプラットフォームバッジ、サポート状況 (support matrix)、インストール手順、アップデート案内の内容を Windows / Linux も対象とした文面に更新する
- [x] 5.2 `docs/development-guide.md` および `docs/development-guide.ja.md` における事前準備 (prerequisites)、ビルド手順、サポートOSに関する記載を更新する
- [x] 5.3 macOS をメインとするメンテナ向けに、Windows や Linux のサポートについて検証するための「検証レーン」を文書化する (参照すべき CI ジョブやログ、手動テストの入り口等の道標を明確にする)
- [x] 5.4 Windows / Linux それぞれに向け、VM やリモートマシン、あるいは物理実機のどの環境でも実行できる「ランタイム起動時の動作確認リスト（runtime smoke checklist）」を作成し、初回起動、ワークスペースオープン、Markdown の編集・プレビュー表示に関する必須確認事項 (required evidence) を規定・定義する
- [x] 5.5 リリースブロッカー（リリース不可）となる条件として、Windows / Ubuntu CI の成功、アセットの生成、手動確認事項のクリアが含まれていることを明文化する
- [x] 5.6 OpenSpec の要件定義、設計、タスクの各ドキュメントと実装ファイルの対応関係が崩れていないかを入念に確認する

### Definition of Done (DoD)

- [x] リポジトリ直下の公開用ドキュメント群から、「macOS 専用」といった表現がすべて除去されていること
- [x] 読者から見て、Windows / Linux 向けのインストール、関連ビルド手順、アップデートや検証方法がわかりやすく明示されていること
- [x] Windows / Linux 向けのスモークテスト (runtime smoke checklist) と必須の確認結果エビデンスの要件が明確に文書化されていること
- [x] サポート状況、アセットファイル名、アップデートポリシー、検証手順のドキュメントが、今回の提案・設計・仕様内容と一貫性を保っていること
- [x] `make check` がエラーなし (exit code 0) で通過すること
- [x] `/openspec-delivery` ワークフロー (`.agents/workflows/openspec-delivery.md`) を実行し、包括的なデリバリールーチン (自己レビュー、コミット、PR作成、マージ) を完了すること。

---

## 6. Final Verification & Release Work (最終確認とリリース対応)

- [ ] 6.1 `docs/coding-rules.ja.md` と `.agents/skills/self-review/SKILL.md` に基づく自己レビューを実施する (各ファイルのバージョン情報更新漏れがないか確認する)
- [ ] 6.2 `make check` がエラーなし (exit code 0) で通過することを確認する
- [ ] 6.3 中間ベースブランチ（元々 master から派生したもの）を `master` ブランチへマージする
- [ ] 6.4 `master` に向けて PR（プルリクエスト）を作成する
- [ ] 6.5 `master` へマージする (※ `--admin` の利用は許容される)
- [ ] 6.6 `.agents/skills/release_workflow/SKILL.md` を用いて、`0.18.0` のリリースタグ打ちとリリース作成を実行する
- [ ] 6.7 `/opsx-archive` などの OpenSpec スキルを活用して、このチェンジ全体をアーカイブする
