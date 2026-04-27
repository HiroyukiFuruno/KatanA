const CODE_BLOCK_KIND_COUNT: usize = 17;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodeBlockKind {
    Text,
    Markdown,
    Bash,
    Zsh,
    Mermaid,
    Drawio,
    Plantuml,
    Json,
    Yaml,
    Toml,
    Rust,
    Typescript,
    Javascript,
    Python,
    Html,
    Css,
    Sql,
}

impl CodeBlockKind {
    const ALL: [Self; CODE_BLOCK_KIND_COUNT] = [
        Self::Text,
        Self::Markdown,
        Self::Bash,
        Self::Zsh,
        Self::Mermaid,
        Self::Drawio,
        Self::Plantuml,
        Self::Json,
        Self::Yaml,
        Self::Toml,
        Self::Rust,
        Self::Typescript,
        Self::Javascript,
        Self::Python,
        Self::Html,
        Self::Css,
        Self::Sql,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub fn info_string(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Markdown => "markdown",
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Mermaid => "mermaid",
            Self::Drawio => "drawio",
            Self::Plantuml => "plantuml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Rust => "rust",
            Self::Typescript => "typescript",
            Self::Javascript => "javascript",
            Self::Python => "python",
            Self::Html => "html",
            Self::Css => "css",
            Self::Sql => "sql",
        }
    }

    pub fn display_label(self) -> &'static str {
        self.info_string()
    }
}

/// Operations available via the Markdown authoring command set.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MarkdownAuthoringOp {
    /* WHY: Inline decorations */
    Bold,
    Italic,
    Strikethrough,
    InlineCode,
    /* WHY: Block / structural */
    Heading1,
    Heading2,
    Heading3,
    BulletList,
    NumberedList,
    Blockquote,
    CodeBlock(CodeBlockKind),
    HorizontalRule,
    /* WHY: Reference structures */
    InsertLink,
    InsertTable,
}
