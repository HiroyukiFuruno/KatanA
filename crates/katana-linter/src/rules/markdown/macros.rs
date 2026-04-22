macro_rules! regex_rule {
    ($struct_name:ident, $id:expr, $desc:expr, $regex:expr, $severity:expr) => {
        pub struct $struct_name;
        impl crate::rules::markdown::MarkdownRule for $struct_name {
            fn id(&self) -> &'static str {
                $id
            }
            fn official_meta(&self) -> Option<crate::rules::markdown::OfficialRuleMeta> {
                Some(crate::rules::markdown::OfficialRuleMeta {
                    code: $id,
                    title: stringify!($struct_name),
                    description: $desc,
                    docs_url: concat!(
                        "https://github.com/DavidAnson/markdownlint/blob/main/doc/",
                        $id,
                        ".md"
                    ),
                    parity: crate::rules::markdown::RuleParityStatus::Experimental,
                    is_fixable: false,
                })
            }
            fn evaluate(
                &self,
                file_path: &std::path::Path,
                content: &str,
            ) -> Vec<crate::rules::markdown::MarkdownDiagnostic> {
                let mut diags = Vec::new();
                if let Ok(re) = ::regex::Regex::new($regex) {
                    for (i, line) in content.lines().enumerate() {
                        for cap in re.captures_iter(line) {
                            if let Some(m) = cap.get(0) {
                                diags.push(crate::rules::markdown::MarkdownDiagnostic {
                                    file: file_path.to_path_buf(),
                                    severity: $severity,
                                    range: crate::rules::markdown::DiagnosticRange {
                                        start_line: i + 1,
                                        start_column: m.start() + 1,
                                        end_line: i + 1,
                                        end_column: m.end() + 1,
                                    },
                                    message: $desc.to_string(),
                                    rule_id: $id.to_string(),
                                    official_meta: self.official_meta(),
                                    fix_info: None,
                                });
                            }
                        }
                    }
                }
                diags
            }
        }
    };
}

#[macro_export]
macro_rules! official_rule {
    ($struct_name:ident, $id:expr, $docs_url:expr) => {
        pub struct $struct_name;
        impl $crate::rules::markdown::MarkdownRule for $struct_name {
            fn id(&self) -> &'static str {
                $id
            }
            fn official_meta(&self) -> Option<$crate::rules::markdown::OfficialRuleMeta> {
                Some($crate::rules::markdown::OfficialRuleMeta {
                    code: $id,
                    title: "TBD",
                    description: "TBD",
                    docs_url: $docs_url,
                    parity: $crate::rules::markdown::RuleParityStatus::Official,
                    is_fixable: false,
                })
            }
            fn evaluate(
                &self,
                _file_path: &std::path::Path,
                _content: &str,
            ) -> Vec<$crate::rules::markdown::MarkdownDiagnostic> {
                vec![]
            }
        }
    };
}
