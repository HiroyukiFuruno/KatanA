## Context

Katana を一般ユーザーへ配布するためには、インストーラーとして機能する `.dmg` (Disk Image) 形式での配布が macOS において適しています。さらに継続的な配布を行うため、チェンジログやバージョン管理を自動化した堅牢なリリースフローが必要です。

また、KatanaはOSSとしての公開を前提としています。OSSのパブリックリポジトリにおいて CI/CD（特にリリース・パッケージ関連）の不要な自動実行を放置すると、フォークからのPRによってシークレットを盗取されたり、CIコストを消費させられるリスクがあります。そのため、「安全なOSS公開」に向けた環境とワークフローの権限整理も本変更のスコープに含めます。

### 現状の把握

- **アイコン:** `assets/icon.icns` および `assets/icon.iconset/` は既に存在しており、新規作成は不要。
- **CI:** `.github/workflows/ci.yml` が既に存在し、`permissions: contents: read` の最小権限ポリシー、CodeQL セキュアスキャン等が導入済み。ブランチターゲットは `main`。
- **ドキュメント:** `docs/` に `coding-rules.ja.md` / `coding-rules.md` が既に存在。移動時にこれらとの整合を考慮する必要がある。
- **SECURITY.md**: `.github/SECURITY.md` にセキュリティ報告ポリシーが既にある（OSS前提で整備済み）。

## Goals / Non-Goals

**Goals:**

- `.app` バンドルからインストーラー用 `.dmg` を生成する。
- 開発者が `make release` コマンド等でチェンジログとバージョン、タグ更新を一貫して行えるようにする。
- GitHub Releases による DMG の配布と、一般ユーザー向けを重視した README の再構築を行う。
- **OSS向けCI/CD保護と公開:** 既存の CI ワークフロー（テスト・リント）と分離した CD ワークフローを新設し、リリースパイプラインを「特定ユーザー（オーナー）」のみが発火可能にする。
- パイプライン完成の最終確認前にリポジトリの設定を Private から Public に変更し、OSSとして正式に公開する。

**Non-Goals:**

- Apple Developer Program 経由でのコード署名（Code Signing）と公証（Notarization）の完全自動化。（未署名アプリとして公開）
- macOS 以外のプラットフォーム用インストーラー作成。

## Decisions

- **`.app` バンドルの生成戦略:** `cargo-bundle` の採用を第一候補とするが、メンテナンス状況に懸念があるため、動作しない場合のフォールバックとして、シェルスクリプトによる手動の `.app` ディレクトリ構造構築（`mkdir -p Katana.app/Contents/MacOS` 等 + `Info.plist` テンプレート）も選択肢とし、タスク実施時に検証する。
- **`.dmg` の生成:** `create-dmg`（Homebrew経由）を基本とし、`hdiutil` 単体でのフォールバックも可能とする。
- **配布リポジトリの実装戦略:** 同一リポジトリ (Single Repository) 方式とし、GitHub Releases ページで `.dmg` を提供する。
- **ドキュメント戦略:** ルートの `README.md` / `README.ja.md` はインストール前提の内容へ転換する。開発者向け情報（ビルド手順、コントリビュート案内）は `docs/` に集約する。`docs/` には既存の `coding-rules.ja.md` / `coding-rules.md` があるため、これらは変更せずそのまま残し、新たに `docs/development-guide.md` 等を追加する形をとる。
- **CI/CDの権限モデル:**
  既存の `ci.yml`（テスト・リント・CodeQL・カバレッジ）はそのまま維持する（`permissions: contents: read` の最小権限ポリシーが既に適用済み）。リリース・パッケージング用の CD ワークフローを**別ファイル**（例: `release.yml`）で新設し、`workflow_dispatch` イベントか、タグの push（`v*` パターン）をトリガーとする。加えて、`github.actor` チェックや GitHub Environment の Manual Approvers 設定によりオーナーのみ実行可能にする。
- **ブランチ名:** 既存CIは `main` ブランチをターゲットとしている。tasks では `main` をベースブランチとして使用する（`master` ではなく `main`）。

## Risks / Trade-offs

- **Risk:** `cargo-bundle` がメンテナンス不全で動作しない、または最新 Rust ツールチェインと互換性がない。
- **Mitigation:** タスク実施時に動作確認を行い、不具合があればシェルスクリプトでの `.app` 構築にフォールバックする。どちらのアプローチでも `.app` の構造は同一であるため、成果物への影響はない。
- **Risk:** サードパーティからのPRなどでCIが通るか確認しにくくなる（権限を絞りすぎる）。
- **Mitigation:** 既存の CI（テスト・リント）は `pull_request` イベントで誰からでも実行可能な状態を維持する（シークレットへのアクセスなし）。DMGビルドやRelease作成処理など「リリース関連のCDフェーズ」のみを既存CIとは別のワークフローに分離し、当該ワークフローのみにオーナー権限による明示的発火制約を設ける。
- **Risk:** 未署名の `.dmg` が Gatekeeper 警告を受ける。
- **Mitigation:** README やリリースノートで「右クリック → 開く」または `xattr -cr` での回避手順を案内する。
