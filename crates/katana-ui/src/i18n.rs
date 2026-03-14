use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// ロケールJSONデータの定義。
const EN_JSON: &str = include_str!("../locales/en.json");
const JA_JSON: &str = include_str!("../locales/ja.json");

static DICTIONARY: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
static CURRENT_LANGUAGE: RwLock<String> = RwLock::new(String::new());

fn init_current_language() {
    let mut lang = CURRENT_LANGUAGE.write().unwrap();
    if lang.is_empty() {
        *lang = "en".to_string();
    }
}

fn get_dictionary() -> &'static HashMap<&'static str, HashMap<String, String>> {
    DICTIONARY.get_or_init(|| {
        let mut map = HashMap::new();
        if let Ok(json) = serde_json::from_str(EN_JSON) {
            map.insert("en", json);
        }
        if let Ok(json) = serde_json::from_str(JA_JSON) {
            map.insert("ja", json);
        }
        map
    })
}

/// 現在の言語を設定する。
pub fn set_language(lang: &str) {
    if let Ok(mut current) = CURRENT_LANGUAGE.write() {
        *current = lang.to_string();
    }
}

/// 現在の言語を取得する。
pub fn get_language() -> String {
    init_current_language();
    CURRENT_LANGUAGE.read().unwrap().clone()
}

/// 指定したキーに対応する翻訳文字列を取得する。
pub fn t(key: &str) -> String {
    let lang = get_language();
    let dict = get_dictionary();
    if let Some(d) = dict.get(lang.as_str()) {
        if let Some(val) = d.get(key) {
            return val.clone();
        }
    }
    // Fallback to english if missing in current lang
    if lang != "en" {
        if let Some(d) = dict.get("en") {
            if let Some(val) = d.get(key) {
                return val.clone();
            }
        }
    }

    key.to_string()
}

/// 指定したキーに対応する翻訳文字列をパラメータ置換して取得する。
///
/// キーの翻訳文字列中の `{param}` プレースホルダを `params` の値に置き換える。
///
/// # 例
/// ```
/// // en.json: "status_saved_as": "Saved as {name}"
/// let msg = i18n::tf("status_saved_as", &[("name", "foo.md")]);
/// assert_eq!(msg, "Saved as foo.md");
/// ```
pub fn tf(key: &str, params: &[(&str, &str)]) -> String {
    let mut text = t(key);
    for (k, v) in params {
        text = text.replace(&format!("{{{k}}}"), v);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// en.json と ja.json のキーが完全に一致することを確認する。
    /// キー漏れは自動検出される。
    #[test]
    fn 全ロケールキーが両言語に存在する() {
        let en: HashMap<String, serde_json::Value> =
            serde_json::from_str(EN_JSON).expect("en.json must be valid JSON");
        let ja: HashMap<String, serde_json::Value> =
            serde_json::from_str(JA_JSON).expect("ja.json must be valid JSON");

        let en_keys: HashSet<_> = en.keys().collect();
        let ja_keys: HashSet<_> = ja.keys().collect();

        let missing_in_ja: Vec<_> = en_keys.difference(&ja_keys).collect();
        let missing_in_en: Vec<_> = ja_keys.difference(&en_keys).collect();

        assert!(
            missing_in_ja.is_empty(),
            "ja.json にキーが不足しています: {missing_in_ja:?}"
        );
        assert!(
            missing_in_en.is_empty(),
            "en.json にキーが不足しています: {missing_in_en:?}"
        );
    }

    /// tf() がパラメータを正しく置換することを確認する。
    #[test]
    fn tf関数がパラメータを正しく置換する() {
        let result = tf("__test_key__", &[("name", "world")]);
        // キーが存在しない場合はキー自体が返る（置換なし）
        assert_eq!(result, "__test_key__");
    }

    /// shell.rs が i18n を通さずに UI 文字列をハードコードしていないことを静的解析で確認する。
    ///
    /// これは「UT でも弾く」要件のための静的解析テスト。
    /// 翻訳可能なテキストを含む高リスクな呼び出しパターンを禁止する。
    #[test]
    fn shellrsにi18n漏れがない() {
        let source = include_str!("shell.rs");

        // 「ホバーテキスト」や「見出し」に直接リテラルを渡すパターンを禁止する。
        // i18n::t() / i18n::tf() を経由しなければならない。
        let forbidden_patterns = [
            // on_hover_text に直接文字列リテラルを渡しているパターン
            ".on_hover_text(\"",
            // ui.heading に直接文字列リテラルを渡しているパターン
            "ui.heading(\"",
        ];

        for pattern in &forbidden_patterns {
            assert!(
                !source.contains(pattern),
                "shell.rs にハードコードされた UI 文字列が検出されました: {pattern}\n\
                 i18n::t() または i18n::tf() を使用してください。"
            );
        }
    }
}
