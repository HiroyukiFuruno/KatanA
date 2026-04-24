## Why

現在、KatanAのUI (`katana-ui`) は、Markdownのパース・プレビュー描画（Mermaid等ダイアグラム描画を含む）とエディタ機能をすべて抱え込んでいます（Fat UI）。特に、プレビューに関連する `egui_commonmark` 等のフォークを KatanA ルートの `vendor/` フォルダで直接管理し `[patch.crates-io]` で当てている状態は、リポジトリ境界のクリーンさを著しく損ねています。
テスト容易性、ビルド速度の向上、そして依存関係のクリーン化を図るため、第一段階（v0.26.0）としてプレビュー機能を独立したクレート（`katana-markdown-preview`）に分離し、`vendor/` からの脱却を図ります。

## What Changes

- `katana-markdown-preview` クレートの切り出し：Markdownテキストを受け取って `egui` 上にネイティブ描画する責務をカプセル化（WebViewは使用せず、純粋なRustネイティブを維持）。
- `vendor/` のカプセル化と整理：`egui_commonmark` や関連するフォークの依存をこのクレート内に隠蔽し、必要に応じて別リポジトリ（Git参照）化してクリーンアップ。
- ネイティブでの絵文字対応（ハック）：`egui` のモノクロ描画制限を突破するため、テキストパース時に絵文字を抽出して `egui::Image` (Twemoji等のアセット) として置換・描画するネイティブハックを導入。
- `katana-ui` のプレビュー責務剥奪：描画詳細のロジックを排除し、プレビューウィジェットを利用して配置する「糊」としての役割へ特化。

## Capabilities

### New Capabilities
- `markdown-preview-component`: Markdownパース、ダイアグラム描画、テーブル対応をラップしたプレビューウィジェット。

### Modified Capabilities
- `ui-architecture`: Fat UI から「コンポーネントの組み立て」へ特化する UI アーキテクチャの変更（プレビュー分離版）。

## Impact

- `crates/katana-ui`: プレビューに関する大規模なコード削除とコンポーネント利用への置き換え。
- ルート `Cargo.toml`: `[patch.crates-io]` および `vendor/` の削除。
- ワークスペース構成: 新規クレート (`katana-markdown-preview`) の追加。
- CI/CD: 独立したインテグレーションテストが可能になるため、CIの並列実行効率が向上。
