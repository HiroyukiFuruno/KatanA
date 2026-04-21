# 改善提案: i18n対応の高度化 (不適切なi18n対応)

## 不適切なi18n対応: 欧州言語における `task_todo` の未翻訳・不自然な翻訳
`task_todo` の翻訳文字列が、英語では "Task [ ]" であるのに対し、ドイツ語 (`de`)、スペイン語 (`es`)、ポルトガル語 (`pt`)、イタリア語 (`it`) などの複数の欧州言語において `"To-Do [ ]"` のままとなっています。
また、Linterのソースコード（`crates/katana-linter/src/rules/domains/locales/values.rs`）にも擬似翻訳(`[TODO`)を防ぐロジックがありますが、ネイティブな翻訳者が確認し、各言語の文脈に沿った自然な言い回しに修正する必要があります。

## 不適切なi18n対応: 単純すぎるテンプレート置換ロジック (`I18nOps::tf`)
`crates/katana-ui/src/i18n/logic.rs` に実装されている文字列補間関数 `tf` は、単なる `{key}` の `.replace()` に依存しています。
```rust
pub fn tf(template: &str, params: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}
```
この実装は単純な置き換えには機能しますが、フランス語やスペイン語などで頻出する「複数形（Pluralization）の分岐」や「性別（Gender）による活用変化」を扱うことができません。（例: 1 problem vs 2 problems）
本格的なデスクトップアプリケーションとしてグローバル展開を強化するため、`fluent-rs` のようなより高度なi18nフレームワークの導入、または ICUベースのフォーマットエンジンへの移行を提案します。
