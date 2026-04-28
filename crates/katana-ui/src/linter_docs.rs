use std::path::{Path, PathBuf};

const VIRTUAL_DOC_PREFIX: &str = "Katana://LinterDocs/";
const KML_DOCS_RAW_BASE: &str =
    "https://raw.githubusercontent.com/HiroyukiFuruno/katana-markdown-linter/main/upstream_docs";
const KML_DOCS_GITHUB_BASE: &str =
    "https://github.com/HiroyukiFuruno/katana-markdown-linter/blob/main/upstream_docs";
const MARKDOWNLINT_DOCS_GITHUB_BASE: &str =
    "https://github.com/DavidAnson/markdownlint/blob/main/doc";

pub(crate) struct LinterDocIdentity {
    rule_id: String,
    locale_code: String,
}

impl LinterDocIdentity {
    pub(crate) fn for_current_language(rule_id: &str) -> Self {
        Self::for_language(rule_id, &crate::i18n::I18nOps::get_language())
    }

    pub(crate) fn for_language(rule_id: &str, language_code: &str) -> Self {
        let locale = katana_markdown_linter::resolve_locale_code(language_code);
        Self {
            rule_id: normalized_rule_id(rule_id),
            locale_code: locale.code().to_string(),
        }
    }

    pub(crate) fn from_cache_key(cache_key: &str) -> Self {
        let Some((locale_code, rule_id)) = cache_key.split_once(':') else {
            return Self::for_language(cache_key, "en");
        };
        Self {
            rule_id: normalized_rule_id(rule_id),
            locale_code: locale_code.to_string(),
        }
    }

    pub(crate) fn from_virtual_path(path: &Path) -> Option<Self> {
        let raw = path.to_string_lossy();
        let rest = raw.strip_prefix(VIRTUAL_DOC_PREFIX)?;
        let rest = rest.strip_suffix(".md")?;
        if let Some((locale_code, rule_id)) = rest.split_once('/') {
            return Some(Self {
                rule_id: normalized_rule_id(rule_id),
                locale_code: locale_code.to_string(),
            });
        }
        Some(Self::for_language(rest, "en"))
    }

    pub(crate) fn cache_key(&self) -> String {
        format!("{}:{}", self.locale_code, self.rule_id)
    }

    pub(crate) fn virtual_path(&self) -> PathBuf {
        PathBuf::from(format!(
            "{VIRTUAL_DOC_PREFIX}{}/{}.md",
            self.locale_code, self.rule_id
        ))
    }

    pub(crate) fn status_label(&self) -> String {
        format!("{} ({})", self.rule_id, self.locale_code)
    }

    pub(crate) fn raw_url(&self, fallback_docs_url: &str) -> String {
        if self.locale_code == "ja" {
            return format!("{KML_DOCS_RAW_BASE}/ja/{}", self.file_name());
        }
        raw_markdownlint_url(fallback_docs_url)
    }

    pub(crate) fn github_url(&self) -> String {
        if self.locale_code == "ja" {
            return format!("{KML_DOCS_GITHUB_BASE}/ja/{}", self.file_name());
        }
        format!("{MARKDOWNLINT_DOCS_GITHUB_BASE}/{}", self.file_name())
    }

    pub(crate) fn rule_id(&self) -> &str {
        &self.rule_id
    }

    fn file_name(&self) -> String {
        format!("{}.md", self.rule_id.to_ascii_lowercase())
    }
}

fn normalized_rule_id(rule_id: &str) -> String {
    rule_id.to_ascii_uppercase()
}

fn raw_markdownlint_url(docs_url: &str) -> String {
    let raw_url = docs_url
        .replace("github.com", "raw.githubusercontent.com")
        .replace("/blob/", "/");
    lower_file_name(&raw_url)
}

fn lower_file_name(url: &str) -> String {
    let Some(position) = url.rfind('/') else {
        return url.to_ascii_lowercase();
    };
    let (prefix, file_name) = url.split_at(position + 1);
    format!("{}{}", prefix, file_name.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn japanese_rule_doc_uses_kml_localized_markdown() {
        let identity = LinterDocIdentity::for_language("MD001", "ja");

        assert_eq!(
            identity.raw_url("https://github.com/DavidAnson/markdownlint/blob/main/doc/md001.md"),
            "https://raw.githubusercontent.com/HiroyukiFuruno/katana-markdown-linter/main/upstream_docs/ja/md001.md"
        );
        assert_eq!(
            identity.github_url(),
            "https://github.com/HiroyukiFuruno/katana-markdown-linter/blob/main/upstream_docs/ja/md001.md"
        );
    }

    #[test]
    fn english_rule_doc_keeps_markdownlint_source() {
        let identity = LinterDocIdentity::for_language("MD038", "en");

        assert_eq!(
            identity.raw_url("https://github.com/DavidAnson/markdownlint/blob/main/doc/MD038.md"),
            "https://raw.githubusercontent.com/DavidAnson/markdownlint/main/doc/md038.md"
        );
        assert_eq!(
            identity.github_url(),
            "https://github.com/DavidAnson/markdownlint/blob/main/doc/md038.md"
        );
    }

    #[test]
    fn virtual_path_roundtrip_preserves_locale() {
        let identity = LinterDocIdentity::for_language("md001", "ja");
        let parsed = LinterDocIdentity::from_virtual_path(&identity.virtual_path()).unwrap();

        assert_eq!(parsed.cache_key(), "ja:MD001");
    }

    #[test]
    fn current_language_identity_normalizes_rule_id() {
        let identity = LinterDocIdentity::for_current_language("md013");

        assert_eq!(identity.rule_id(), "MD013");
        assert!(identity.status_label().contains("MD013"));
    }

    #[test]
    fn cache_key_parsing_preserves_locale_and_normalizes_rule_id() {
        let identity = LinterDocIdentity::from_cache_key("ja:md024");

        assert_eq!(identity.cache_key(), "ja:MD024");
        assert_eq!(identity.status_label(), "MD024 (ja)");
    }

    #[test]
    fn cache_key_without_locale_falls_back_to_english() {
        let identity = LinterDocIdentity::from_cache_key("md010");

        assert_eq!(identity.cache_key(), "en:MD010");
    }

    #[test]
    fn virtual_path_without_locale_falls_back_to_english() {
        let parsed =
            LinterDocIdentity::from_virtual_path(Path::new("Katana://LinterDocs/md011.md"))
                .unwrap();

        assert_eq!(parsed.cache_key(), "en:MD011");
    }

    #[test]
    fn lower_file_name_without_slash_lowercases_entire_value() {
        assert_eq!(lower_file_name("MD013.MD"), "md013.md");
    }
}
