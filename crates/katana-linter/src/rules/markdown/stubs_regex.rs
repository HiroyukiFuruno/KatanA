use crate::rules::markdown::DiagnosticSeverity;

regex_rule!(
    RuleMD009,
    "MD009",
    "Trailing spaces",
    r" \s+$",
    DiagnosticSeverity::Warning
);
regex_rule!(
    RuleMD010,
    "MD010",
    "Hard tabs",
    r"\t",
    DiagnosticSeverity::Warning
);
regex_rule!(
    RuleMD018,
    "MD018",
    "No space after hash on atx style heading",
    r"^#+[^\s#]",
    DiagnosticSeverity::Error
);
regex_rule!(
    RuleMD019,
    "MD019",
    "Multiple spaces after hash on atx style heading",
    r"^#+\s{2,}[^\s#]",
    DiagnosticSeverity::Warning
);
regex_rule!(
    RuleMD037,
    "MD037",
    "Spaces inside emphasis markers",
    r"(?:\*\*|__|\*|_)\s+[^\s].*[^\s]\s+(?:\*\*|__|\*|_)",
    DiagnosticSeverity::Warning
);
regex_rule!(
    RuleMD038,
    "MD038",
    "Spaces inside code span elements",
    r"`\s+[^`]+\s+`",
    DiagnosticSeverity::Warning
);
regex_rule!(
    RuleMD039,
    "MD039",
    "Spaces inside link text",
    r"\[\s+[^\]]+\s+\]",
    DiagnosticSeverity::Warning
);
