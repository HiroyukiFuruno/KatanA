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
    CodeBlock,
    HorizontalRule,
    /* WHY: Reference structures */
    InsertLink,
    InsertTable,
}
