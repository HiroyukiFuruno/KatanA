macro_rules! regex_rule {
    ($struct_name:ident, $id:expr, $desc:expr, $regex:expr, $severity:expr) => {
        crate::regex_rule!($struct_name, $id, $desc, $regex, $severity, &[]);
    };
    ($struct_name:ident, $id:expr, $desc:expr, $regex:expr, $severity:expr, $properties:expr) => {
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
                    properties: $properties,
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
        $crate::official_rule!($struct_name, $id, "TBD", "TBD", $docs_url, &[]);
    };
    ($struct_name:ident, $id:expr, $docs_url:expr, $properties:expr) => {
        $crate::official_rule!($struct_name, $id, "TBD", "TBD", $docs_url, $properties);
    };
    ($struct_name:ident, $id:expr, $title:expr, $desc:expr, $docs_url:expr, $properties:expr) => {
        pub struct $struct_name;
        impl $crate::rules::markdown::MarkdownRule for $struct_name {
            fn id(&self) -> &'static str {
                $id
            }
            fn official_meta(&self) -> Option<$crate::rules::markdown::OfficialRuleMeta> {
                Some($crate::rules::markdown::OfficialRuleMeta {
                    code: $id,
                    title: $title,
                    description: $desc,
                    docs_url: $docs_url,
                    parity: $crate::rules::markdown::RuleParityStatus::Official,
                    is_fixable: false,
                    properties: $properties,
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

#[macro_export]
macro_rules! rule_prop {
    ($pt:ident, $k:expr, $d:expr, $def:expr) => {
        $crate::rules::markdown::RuleProperty {
            key: $k,
            prop_type: $crate::rules::markdown::RulePropertyType::$pt,
            description: $d,
            default_value: $def,
        }
    };
}

#[macro_export]
macro_rules! rule_prop_enum {
    ($k:expr, $d:expr, $def:expr, $options:expr) => {
        $crate::rules::markdown::RuleProperty {
            key: $k,
            prop_type: $crate::rules::markdown::RulePropertyType::Enum($options),
            description: $d,
            default_value: $def,
        }
    };
}
