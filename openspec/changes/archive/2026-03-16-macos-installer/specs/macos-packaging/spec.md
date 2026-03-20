## ADDED Requirements

### Requirement: macOS アプリケーションバンドル (.app) の生成
システムは、`cargo-bundle` またはシェルスクリプトのいずれかのアプローチにより、標準の macOS `.app` バンドル構造（`Contents/MacOS/`, `Contents/Resources/` など）を構築できなければならない（MUST）。既存の `assets/icon.icns` を同梱すること。

#### Scenario: .app バンドルのビルド
- **WHEN** 開発者が `make package-mac`（またはその内部処理に対応するコマンド）を実行したとき
- **THEN** メタデータと既存のアイコン画像 (`assets/icon.icns`) が組み込まれた `Katana.app` バンドルが `target/` 配下等の所定の位置に正しく生成される。

### Requirement: ディスクイメージ (.dmg) の構築
生成された `.app` ファイルは、ユーザーがドラッグ＆ドロップで容易にインストールできるよう、`.dmg` 形式で再パッケージされなければならない（MUST）。

#### Scenario: .dmg ディスクイメージのビルド
- **WHEN** `Katana.app` が構築されたのち、開発者が `make dmg`（またはそれに準ずるコマンド）を実行したとき
- **THEN** Applications へのシンボリックリンクを含んだ配布用の `Katana-<version>.dmg` が `target/` ディレクトリ等の所定の位置に生成され、マウント可能な状態になる。
