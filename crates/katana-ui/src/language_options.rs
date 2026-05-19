pub(crate) struct LanguageOption {
    pub(crate) code: &'static str,
    pub(crate) label: String,
}

pub(crate) struct LanguageOptionOps;

impl LanguageOptionOps {
    pub(crate) fn menu_options(i18n: &crate::i18n::I18nMessages) -> Vec<LanguageOption> {
        vec![
            Self::option(
                katana_platform::settings::AUTO_LANGUAGE_CODE,
                &i18n.menu.language_auto,
            ),
            Self::option("en", &i18n.menu.language_en),
            Self::option("ja", &i18n.menu.language_ja),
            Self::option("zh-CN", &i18n.menu.language_zh_cn),
            Self::option("zh-TW", &i18n.menu.language_zh_tw),
            Self::option("ko", &i18n.menu.language_ko),
            Self::option("pt", &i18n.menu.language_pt),
            Self::option("fr", &i18n.menu.language_fr),
            Self::option("de", &i18n.menu.language_de),
            Self::option("es", &i18n.menu.language_es),
            Self::option("it", &i18n.menu.language_it),
        ]
    }

    pub(crate) fn label_for(code: &str, i18n: &crate::i18n::I18nMessages) -> String {
        Self::menu_options(i18n)
            .into_iter()
            .find(|it| it.code == code)
            .map(|it| it.label)
            .unwrap_or_else(|| code.to_string())
    }

    fn option(code: &'static str, label: &str) -> LanguageOption {
        LanguageOption {
            code,
            label: label.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_options_puts_auto_language_first() {
        let i18n = crate::i18n::I18nOps::get();

        let options = LanguageOptionOps::menu_options(i18n);

        assert_eq!(
            options.first().map(|it| it.code),
            Some(katana_platform::settings::AUTO_LANGUAGE_CODE)
        );
        assert_eq!(
            options.first().map(|it| it.label.as_str()),
            Some(i18n.menu.language_auto.as_str())
        );
        assert_eq!(options.len(), 11);
    }

    #[test]
    fn label_for_resolves_known_code_and_keeps_unknown_code() {
        let i18n = crate::i18n::I18nOps::get();

        assert_eq!(
            LanguageOptionOps::label_for("ja", i18n),
            i18n.menu.language_ja
        );
        assert_eq!(
            LanguageOptionOps::label_for("custom-lang", i18n),
            "custom-lang"
        );
    }
}
