



official_rule!(RuleMD001, "MD001", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md001.md", &[
    crate::rule_prop!(String, "front_matter_title", "RegExp for matching title in front matter", "^\\s*title\\s*[:=]"),
]);
official_rule!(RuleMD003, "MD003", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md003.md", &[
    crate::rule_prop_enum!("style", "Heading style", "consistent", &["consistent", "atx", "atx_closed", "setext", "setext_with_atx", "setext_with_atx_closed"]),
]);
official_rule!(RuleMD004, "MD004", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md004.md", &[
    crate::rule_prop_enum!("style", "List style", "consistent", &["consistent", "asterisk", "plus", "dash", "sublist"]),
]);
official_rule!(RuleMD005, "MD005", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md005.md", &[]);
official_rule!(RuleMD007, "MD007", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md007.md", &[
    crate::rule_prop!(Number, "indent", "Spaces for indent", "2"),
    crate::rule_prop!(Boolean, "start_indented", "Whether to indent the first level of the list", "false"),
    crate::rule_prop!(Number, "start_indent", "Spaces for first level indent (when start_indented is set)", "2"),
]);
official_rule!(RuleMD011, "MD011", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md011.md", &[]);
official_rule!(RuleMD012, "MD012", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md012.md", &[
    crate::rule_prop!(Number, "maximum", "Consecutive blank lines", "1"),
]);
official_rule!(RuleMD013, "MD013", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md013.md", &[
    crate::rule_prop!(Number, "line_length", "Number of characters", "80"),
    crate::rule_prop!(Number, "heading_line_length", "Number of characters for headings", "80"),
    crate::rule_prop!(Number, "code_block_line_length", "Number of characters for code blocks", "80"),
    crate::rule_prop!(Boolean, "code_blocks", "Include code blocks", "true"),
    crate::rule_prop!(Boolean, "tables", "Include tables", "true"),
    crate::rule_prop!(Boolean, "headings", "Include headings", "true"),
    crate::rule_prop!(Boolean, "strict", "Strict length checking", "false"),
    crate::rule_prop!(Boolean, "stern", "Stern length checking", "false"),
]);
official_rule!(RuleMD014, "MD014", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md014.md", &[]);
official_rule!(RuleMD020, "MD020", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md020.md", &[]);
official_rule!(RuleMD021, "MD021", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md021.md", &[]);
official_rule!(RuleMD022, "MD022", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md022.md", &[
    crate::rule_prop!(Number, "lines_above", "Blank lines above heading", "1"),
    crate::rule_prop!(Number, "lines_below", "Blank lines below heading", "1"),
]);
official_rule!(RuleMD023, "MD023", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md023.md", &[]);
official_rule!(RuleMD024, "MD024", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md024.md", &[
    crate::rule_prop!(Boolean, "siblings_only", "Only check sibling headings", "false"),
]);
official_rule!(RuleMD025, "MD025", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md025.md", &[
    crate::rule_prop!(String, "front_matter_title", "RegExp for matching title in front matter", "^\\s*title\\s*[:=]"),
    crate::rule_prop!(Number, "level", "Heading level", "1"),
]);
official_rule!(RuleMD026, "MD026", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md026.md", &[
    crate::rule_prop!(String, "punctuation", "Punctuation characters", ".,;:!。，；：！"),
]);
official_rule!(RuleMD027, "MD027", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md027.md", &[
    crate::rule_prop!(Boolean, "list_items", "Include list items", "true"),
]);
official_rule!(RuleMD028, "MD028", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md028.md", &[]);
official_rule!(RuleMD029, "MD029", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md029.md", &[
    crate::rule_prop_enum!("style", "List style", "one_or_ordered", &["one", "ordered", "one_or_ordered", "zero"]),
]);
official_rule!(RuleMD030, "MD030", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md030.md", &[
    crate::rule_prop!(Number, "ul_single", "Spaces for single-line unordered list items", "1"),
    crate::rule_prop!(Number, "ol_single", "Spaces for single-line ordered list items", "1"),
    crate::rule_prop!(Number, "ul_multi", "Spaces for multi-line unordered list items", "1"),
    crate::rule_prop!(Number, "ol_multi", "Spaces for multi-line ordered list items", "1"),
]);
official_rule!(RuleMD031, "MD031", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md031.md", &[
    crate::rule_prop!(Boolean, "list_items", "Include list items", "true"),
]);
official_rule!(RuleMD032, "MD032", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md032.md", &[]);
official_rule!(RuleMD033, "MD033", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md033.md", &[
    crate::rule_prop!(StringArray, "allowed_elements", "Allowed elements", "[]"),
    crate::rule_prop!(StringArray, "table_allowed_elements", "Allowed elements in tables", "[]"),
]);
official_rule!(RuleMD034, "MD034", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md034.md", &[]);
official_rule!(RuleMD035, "MD035", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md035.md", &[
    crate::rule_prop_enum!("style", "Horizontal rule style", "consistent", &["consistent", "atx", "atx_closed", "setext", "setext_with_atx", "setext_with_atx_closed"]),
]);
official_rule!(RuleMD036, "MD036", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md036.md", &[
    crate::rule_prop!(String, "punctuation", "Punctuation characters", ".,;:!?。，；：！？"),
]);
official_rule!(RuleMD040, "MD040", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md040.md", &[
    crate::rule_prop!(StringArray, "allowed_languages", "List of languages", "[]"),
    crate::rule_prop!(Boolean, "language_only", "Require language only", "false"),
]);
official_rule!(RuleMD041, "MD041", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md041.md", &[
    crate::rule_prop!(Boolean, "allow_preamble", "Allow content before first heading", "false"),
    crate::rule_prop!(String, "front_matter_title", "RegExp for matching title in front matter", "^\\s*title\\s*[:=]"),
    crate::rule_prop!(Number, "level", "Heading level", "1"),
]);
official_rule!(RuleMD042, "MD042", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md042.md", &[]);
official_rule!(RuleMD043, "MD043", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md043.md", &[
    crate::rule_prop!(StringArray, "headings", "List of headings", "[]"),
    crate::rule_prop!(Boolean, "match_case", "Match case of headings", "false"),
]);
official_rule!(RuleMD044, "MD044", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md044.md", &[
    crate::rule_prop!(StringArray, "names", "List of proper names", "[]"),
    crate::rule_prop!(Boolean, "code_blocks", "Include code blocks", "true"),
    crate::rule_prop!(Boolean, "html_elements", "Include HTML elements", "true"),
]);
official_rule!(RuleMD045, "MD045", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md045.md", &[]);
official_rule!(RuleMD046, "MD046", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md046.md", &[
    crate::rule_prop_enum!("style", "Block style", "consistent", &["consistent", "fenced", "indented"]),
]);
official_rule!(RuleMD047, "MD047", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md047.md", &[]);
official_rule!(RuleMD048, "MD048", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md048.md", &[
    crate::rule_prop_enum!("style", "Code fence style", "consistent", &["consistent", "backtick", "tilde"]),
]);
official_rule!(RuleMD049, "MD049", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md049.md", &[
    crate::rule_prop_enum!("style", "Emphasis style", "consistent", &["consistent", "asterisk", "underscore"]),
]);
official_rule!(RuleMD050, "MD050", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md050.md", &[
    crate::rule_prop_enum!("style", "Strong style", "consistent", &["consistent", "asterisk", "underscore"]),
]);
official_rule!(RuleMD051, "MD051", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md051.md", &[
    crate::rule_prop!(Boolean, "ignore_case", "Ignore case of fragments", "false"),
    crate::rule_prop!(String, "ignored_pattern", "Pattern for ignoring additional fragments", ""),
]);
official_rule!(RuleMD052, "MD052", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md052.md", &[
    crate::rule_prop!(StringArray, "ignored_labels", "Ignored link labels", "[\"x\"]"),
    crate::rule_prop!(Boolean, "shortcut_syntax", "Include shortcut syntax", "false"),
]);
official_rule!(RuleMD053, "MD053", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md053.md", &[
    crate::rule_prop!(StringArray, "ignored_definitions", "Ignored definitions", "[\"//\"]"),
]);
official_rule!(RuleMD054, "MD054", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md054.md", &[
    crate::rule_prop!(Boolean, "autolink", "Allow autolinks", "true"),
    crate::rule_prop!(Boolean, "inline", "Allow inline links and images", "true"),
    crate::rule_prop!(Boolean, "full", "Allow full reference links and images", "true"),
    crate::rule_prop!(Boolean, "collapsed", "Allow collapsed reference links and images", "true"),
    crate::rule_prop!(Boolean, "shortcut", "Allow shortcut reference links and images", "true"),
    crate::rule_prop!(Boolean, "url_inline", "Allow URLs as inline links", "true"),
]);
official_rule!(RuleMD055, "MD055", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md055.md", &[
    crate::rule_prop_enum!("style", "Table pipe style", "consistent", &["consistent", "leading_only", "trailing_only", "leading_and_trailing", "no_leading_or_trailing"]),
]);
official_rule!(RuleMD056, "MD056", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md056.md", &[]);
official_rule!(RuleMD058, "MD058", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md058.md", &[]);
official_rule!(RuleMD059, "MD059", "https://github.com/DavidAnson/markdownlint/blob/main/doc/md059.md", &[
    crate::rule_prop!(StringArray, "prohibited_texts", "Prohibited link texts", "[\"click here\",\"here\",\"link\",\"more\"]"),
]);

