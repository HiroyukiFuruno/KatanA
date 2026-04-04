use crate::utils::LinterParserOps;

pub struct I18nHelperOps;

impl I18nHelperOps {
    pub fn ui_methods() -> Vec<&'static str> {
        vec![
            "label",
            "heading",
            "button",
            "on_hover_text",
            "selectable_label",
            "checkbox",
            "radio",
            "radio_value",
            "small_button",
            "text_edit_singleline",
            "hyperlink_to",
            "collapsing",
            "hint_text",
            "menu_button",
            "with_new_rect",
        ]
    }

    pub fn ui_functions() -> Vec<&'static str> {
        vec!["new"]
    }

    pub fn ui_types_for_new() -> Vec<&'static str> {
        vec!["RichText", "Button", "Window"]
    }

    pub fn is_format_macro(mac: &syn::Macro) -> bool {
        mac.path
            .segments
            .last()
            .map(|it| it.ident == "format")
            .unwrap_or(false)
    }

    pub fn extract_type_from_call(func: &syn::Expr) -> Option<String> {
        if let syn::Expr::Path(expr_path) = func {
            let segments = &expr_path.path.segments;
            if segments.len() >= 2 {
                return Some(segments[segments.len() - 2].ident.to_string());
            }
        }
        None
    }

    pub fn is_raw_icon(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed == "x" || trimmed == "X" {
            return true;
        }
        trimmed.chars().any(LinterParserOps::is_emoji_or_symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_emoji_or_symbol_tag_range() {
        assert!(LinterParserOps::is_emoji_or_symbol('\u{E0001}'));
        assert!(LinterParserOps::is_emoji_or_symbol('\u{E007F}'));
    }
}
