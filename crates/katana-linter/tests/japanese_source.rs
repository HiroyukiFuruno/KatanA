use proc_macro2::{Group, TokenStream, TokenTree};
use std::path::Path;
use std::str::FromStr;

pub(crate) fn is_japanese(character: char) -> bool {
    matches!(character, '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}')
}

pub(crate) fn scan(path: &Path, source: &str) -> Vec<String> {
    if let Err(error) = syn::parse_file(source) {
        return vec![format!(
            "{}:1: Malformed Rust source cannot be checked for Japanese text: {error}",
            path.display()
        )];
    }
    let tokens = match TokenStream::from_str(source) {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![format!(
                "{}:1: Malformed Rust tokens cannot be checked for Japanese text: {error}",
                path.display()
            )];
        }
    };

    let mut violations = Vec::new();
    scan_tokens(path, tokens, &mut violations);
    violations
}

fn scan_tokens(path: &Path, tokens: TokenStream, violations: &mut Vec<String>) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        if let TokenTree::Punct(punct) = &tokens[index]
            && punct.as_char() == '#'
        {
            let group_index = if matches!(tokens.get(index + 1), Some(TokenTree::Punct(punct)) if punct.as_char() == '!')
            {
                index + 2
            } else {
                index + 1
            };
            if let Some(TokenTree::Group(group)) = tokens.get(group_index)
                && group.delimiter() == proc_macro2::Delimiter::Bracket
                && is_doc_attribute(group)
            {
                index = group_index + 1;
                continue;
            }
        }

        match &tokens[index] {
            TokenTree::Group(group) => scan_tokens(path, group.stream(), violations),
            TokenTree::Ident(ident) => {
                check_text(path, ident.to_string(), ident.span(), violations)
            }
            TokenTree::Literal(literal) => {
                check_text(path, literal.to_string(), literal.span(), violations)
            }
            TokenTree::Punct(_) => {}
        }
        index += 1;
    }
}

fn is_doc_attribute(group: &Group) -> bool {
    let Ok(syn::Meta::NameValue(meta)) = syn::parse2(group.stream()) else {
        return false;
    };
    meta.path.is_ident("doc")
        && matches!(
            meta.value,
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(_),
                ..
            })
        )
}

fn check_text(path: &Path, text: String, span: proc_macro2::Span, violations: &mut Vec<String>) {
    if text.chars().any(is_japanese) {
        violations.push(format!(
            "{}:{}: Please remove Japanese text or use Unicode escapes for test strings.",
            path.display(),
            span.start().line
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::scan;
    use std::path::Path;

    fn kana() -> String {
        "\u{304b}\u{306a}".to_owned()
    }

    #[test]
    fn allows_japanese_comments_and_doc_attributes() {
        let kana = kana();
        let source = format!(
            "//! overview {kana}\n/// WHY: explanation {kana}\n#[doc = \"supplement {kana}\"]\nfn main() {{ /* SAFETY: {kana} */ }}\n"
        );
        assert!(scan(Path::new("fixture.rs"), &source).is_empty());
    }

    #[test]
    fn rejects_japanese_code_literals_identifiers_and_strings() {
        let kana = kana();
        let source = format!(
            "fn identifier_{kana}() {{ let _ = \"{kana}\"; let _ = r###\"{kana}\"###; }}\n"
        );
        assert_eq!(scan(Path::new("fixture.rs"), &source).len(), 3);
    }

    #[test]
    fn rejects_comment_looking_text_inside_strings() {
        let kana = kana();
        let source =
            format!("fn main() {{ let _ = \"// {kana}\"; let _ = r##\"/* {kana} */\"##; }}\n");
        assert_eq!(scan(Path::new("fixture.rs"), &source).len(), 2);
    }

    #[test]
    fn allows_unicode_escaped_literals() {
        let source = r#"fn main() { let _ = "\u{304B}"; let _ = r"\u{304B}"; }
"#;
        assert!(scan(Path::new("fixture.rs"), source).is_empty());
    }

    #[test]
    fn non_documentation_attributes_remain_checked() {
        let source = format!("#[deprecated(note = \"{}\")] fn main() {{}}", kana());
        assert_eq!(scan(Path::new("fixture.rs"), &source).len(), 1);
    }

    #[test]
    fn nested_comments_do_not_hide_following_character_literals() {
        let source = format!(
            "/* WHY: outer /* {} */ comment */ fn main() {{ let _ = '\u{304b}'; }}",
            kana()
        );
        assert_eq!(scan(Path::new("fixture.rs"), &source).len(), 1);
    }

    #[test]
    fn chinese_without_kana_keeps_existing_language_policy() {
        let source = format!("fn main() {{ let _ = \"{}\"; }}", "\u{4e2d}\u{6587}");
        assert!(scan(Path::new("fixture.rs"), &source).is_empty());
    }

    #[test]
    fn reports_malformed_rust() {
        let violations = scan(Path::new("fixture.rs"), "fn main( {");
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Malformed Rust source"));
    }
}
