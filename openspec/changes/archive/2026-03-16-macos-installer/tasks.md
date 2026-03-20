## Branch Rule

タスクグループ（##単位）= 1セッションで、以下のサイクルを回す:

1. **ブランチ作成**: `macos-installer-task{N}` を main から切る
2. **実装**: タスクグループ内の全タスクを実装し、DoDを満たす
3. **PR作成**: main に向けてPRを作成する
4. **自己レビュー**: PRの差分をレビューし、問題があれば自己改善する
5. **マージ**: main にマージする

- **命名規則**: `macos-installer-task{N}` (N = グループ番号)
- **ベースブランチ**: main（前タスクのマージ後の最新。既存CIワークフローが `main` をターゲットにしている）
- **セッション運用**: タスクグループごとにセッションを切り替える。各セッションは `/opsx-apply` で開始し、このファイルとdesign.md / specsを参照して実装を進める。

## 1. アプリバンドルのセットアップ

- [x] 1.1 `cargo-bundle` の動作検証を行う。利用不可であれば、シェルスクリプトで `.app` ディレクトリ構造を手動構築するフォールバック方式を採用する。
- [x] 1.2 `crates/katana-ui/Cargo.toml` を更新し、`[package.metadata.bundle]` セクションを追加する（Bundle identifier、既存の `assets/icon.icns` パスの指定など）。`cargo-bundle` を使わない場合はスクリプト側で `Info.plist` テンプレートを用意する。
- [x] 1.3 `Makefile` に新しく `package-mac` ターゲットを追加し、`.app` ファイルが生成できるようにする。

### Definition of Done

- [x] **機能要件**: 既存の `assets/icon.icns` を含むメタデータが正しくビルドに定義され、ローカルで macOS 用のバンドル (`KatanA Desktop.app`) が生成できること。
- [x] プロジェクトルートで `make check-light` が exit 0 で all pass する（fmt-check + lint + 結合テスト + カバレッジ100%維持）
- [x] GitHub CLI 等で `main` に向けて PR を作成する
- [x] 自身の PR をセルフレビューし、必要に応じて自己改善コミットを追加する
- [x] 問題がなければ `main` へマージする（マージ完了をもってタスクグループ完了とする）

## 2. macOS用 DMGの自動構築

- [x] 2.1 DMG作成用ツール (`create-dmg` 等、Homebrew経由) のインストール要件またはスクリプトを準備する。`hdiutil` 単体でのフォールバックも検証する。
- [x] 2.2 `Makefile` に `make dmg` ターゲットを追加し、`package-mac` で生成した `.app` をもとに、Applicationsアイコンへのリンクを含んだ配布用の `KatanA-Desktop-<version>.dmg` が出力されるようにする。
- [x] 2.3 `make dmg` を実行し、生成されたDMGからドラッグ＆ドロップインストールが機能することを確認する。

### Definition of Done

- [x] **機能要件**: `make dmg` コマンド一つでビルドからインストーラー用DMGの生成までが完了し、ユーザーへの配布が可能な状態のバイナリが作れること。
- [x] プロジェクトルートで `make check-light` が exit 0 で all pass する（fmt-check + lint + 結合テスト + カバレッジ100%維持）
- [x] GitHub CLI 等で `main` に向けて PR を作成する
- [x] 自身の PR をセルフレビューし、必要に応じて自己改善コミットを追加する
- [x] 問題がなければ `main` へマージする（マージ完了をもってタスクグループ完了とする）

## 3. リリース自動化（Changelog & Versioning）

- [x] 3.1 チェンジログ生成用ツール (`git-cliff`) をプロジェクトに導入し、設定ファイル (`cliff.toml`) を初期化する。
- [x] 3.2 ワークスペースの全 `Cargo.toml`（ワークスペースルート含む）でバージョンを連動更新するコマンド手順を検証する（`cargo set-version` 等またはシェルスクリプト）。
- [x] 3.3 `Makefile` に `make release VERSION=x.y.z` のようなリリース用ターゲットを追加する。
- [x] 3.4 ターゲットの中で、バージョン書き換え → チェンジログの更新生成 → `git commit` → `git tag v<version>` が通る一連のフローを構築する。
- [x] 3.5 ファーストリリース（`v0.0.1`、過去にタグが一つもない状態）でもチェンジログが正しく生成されること（全コミット履歴が対象になること）を検証する。
- [x] 3.6 テストブランチなどでのリハーサルを通して、一連のパイプライン（DMG作成まで）が正しく連動するか確認する。

### Definition of Done

- [x] **機能要件**: バージョン更新作業やリリースノート（CHANGELOG）の整備、Gitタグ打ちの煩雑な作業が自動化・一本化され、スムーズにバージョンをインクリメントして配布可能になっていること。
- [x] プロジェクトルートで `make check-light` が exit 0 で all pass する（fmt-check + lint + 結合テスト + カバレッジ100%維持）
- [x] GitHub CLI 等で `main` に向けて PR を作成する
- [x] 自身の PR をセルフレビューし、必要に応じて自己改善コミットを追加する
- [x] 問題がなければ `main` へマージする（マージ完了をもってタスクグループ完了とする）

## 4. ドキュメントの一般配布向け再構築

- [x] 4.1 既存のルートにある開発者向けの `README.md` および `README.ja.md` の内容を `docs/development-guide.md` / `docs/development-guide.ja.md` 等として `docs/` ディレクトリ配下に移動・分離する（既存の `docs/coding-rules*.md` はそのまま維持）。
- [x] 4.2 一般ユーザーをターゲットとした新しい `README.md`（および `README.ja.md`）をルートディレクトリに作成する。ベース素材として `katana_readme_v0.0.1.md` の文章を含めること。加えて以下の要素を含めること:
  - アプリケーションアイコン画像の表示（`assets/icon.iconset/` 内の適切なサイズ画像を利用）
  - Shields.io によるバッジ群（ライセンス、CIビルドステータス、最新バージョンタグ、プラットフォーム対応等）
  - 現在のバージョン番号の表示
  - アプリの機能紹介
  - GitHub Releases からの DMG のダウンロード方法
- [x] 4.3 未署名アプリ特有の「右クリック → 開く」または `xattr -cr` コマンドによる初回起動時のセキュリティ制限回避方法を `README` に明記する。
- [x] 4.4 `README` に、開発者としてプロジェクトに参加・ビルドしたい人向けに `docs/` へ誘導するリンクを設置する。

### Definition of Done

- [x] **機能要件**: 開発と運用の視点が分離され、リポジトリに訪れた一般ユーザーが迷うことなく DMG をダウンロードし、インストールから起動まで進められる案内が完成していること。
- [x] プロジェクトルートで `make check-light` が exit 0 で all pass する
- [x] GitHub CLI 等で `main` に向けて PR を作成する
- [x] 自身の PR をセルフレビューし、必要に応じて自己改善コミットを追加する
- [x] 問題がなければ `main` へマージする（マージ完了をもってタスクグループ完了とする）

## 5. OSS化 / CI・CD権限分離とパブリック移行

- [x] 5.1 既存の `.github/workflows/ci.yml`（テスト・リント・CodeQL・カバレッジ）はそのまま維持し、リリース用の CD ワークフロー（例: `.github/workflows/release.yml`）を別ファイルで新設する。
- [x] 5.2 CD ワークフローのトリガーを `workflow_dispatch`（手動実行）または `push tags: ['v*']` とし、処理内で `github.actor == 'hiroyuki-furuno'` の条件チェック、または GitHub Environment（Manual Approvers）を設定してオーナー以外の実行を防止する。
- [x] 5.3 フォークされたリポジトリからの Pull Request によって CD パイプラインが実行されない（既存 CI のみが走る）状態を確認する。
- [x] 5.4 CD ワークフロー内で `permissions: contents: write` を設定し、`KatanA Desktop.app` + `.dmg` の GitHub Releases アップロードが動作することをテストする。
- [ ] 5.5 リポジトリの Visibility を「Private」から「Public」に変更（OSSとして公開）する。 ※ オーナーによる手動操作

### Definition of Done

- [x] **機能要件**: CI（テスト・リント）と CD（リリース・DMGアップロード）のワークフローが分離され、CDはオーナーのみ実行可能な権限モデルとなっていること。リポジトリが Public へ変更されてオープンに公開・利用可能な状態になっていること。
- [x] サードパーティを想定したテスト実行によるCDパイプラインのブロック（Skipped状態）が検証されていること
- [x] オーナー自身の権限（かつ Public化後）においてのみ、正常に DMG までの自動リリースが完了することを実証すること
- [x] 全てのローカル実行においても `make check-light` が exit 0 で all pass すること
- [x] 自身の PR をセルフレビューし、問題がないことを確認の後に `main` へマージ・完結する
