# i18n フレームワーク移行の設計

## 1. アーキテクチャ概略

現在の `katana-ui` の i18n 処理フローを以下のように変更します。

### 現状 (As-Is)

- 起動時に `include_str!` で JSON を読み込み。
- `serde_json` で `I18nMessages` 構造体にデシリアライズ。
- `tf` 内で `String::replace` を実行。

### 移行後 (To-Be)

- 起動時に各言語の `.ftl` ファイルを `FluentBundle` としてロード。
- メッセージ参照は、構造体のフィールド経由ではなく、メッセージID（キー）によるルックアップに変更（または、マクロを使用して型安全なラッパーを生成）。
- `tf` 関数は、`FluentArgs` をラップしたインターフェースを提供。

## 2. 採用技術

- **fluent-rs**: Mozilla が開発した Project Fluent の Rust 実装。非対称な翻訳（言語によってルールが異なる）を扱うのに最適。
- **unic-langid**: 言語識別子の厳密な管理。

## 3. データ移行

1. 既存の JSON 構造をフラットなキー名（例: `menu.file`）に変換するコンバータを作成。
2. 各ロケールの FTL ファイルを生成。
3. 複数形が必要なメッセージ（例: `search.results_count`）について、FTL 側で分岐ルールを記述。

## 4. インターフェース設計 (Rust)

```rust
pub struct I18nOps;

impl I18nOps {
    // 従来の単純置換の代替
    pub fn t(key: &str) -> String;

    // 高度な補間（複数形対応）
    pub fn tf(key: &str, args: impl Into<FluentArgs>) -> String;
}
```

## 5. 移行の段階的戦略

1. **Phase 1**: `fluent-rs` のランタイムを導入し、既存の JSON ベースの読み込みと並行稼働させる。
2. **Phase 2**: ロケールファイルを JSON から FTL に完全移行し、`I18nMessages` 構造体（JSON用）を廃止。
3. **Phase 3**: コード内の `tf` 呼び出し箇所を順次、引数付きの新 API に移行。
