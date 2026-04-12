use super::authoring_utils::AuthoringUtils;
use super::types::AuthoringTransform;
use crate::app_action::MarkdownAuthoringOp;

pub struct MarkdownAuthoringOps;

impl MarkdownAuthoringOps {
    /// Apply a `MarkdownAuthoringOp` to the buffer at the given selection range (byte offsets).
    ///
    /// When `sel_start == sel_end` the cursor is at a point; a snippet is inserted.
    /// When there is a selection, the selected text is wrapped / prefixed.
    ///
    /// Returns the transformed buffer and the resulting cursor byte range.
    pub fn apply(
        buffer: &str,
        sel_start: usize,
        sel_end: usize,
        op: MarkdownAuthoringOp,
    ) -> AuthoringTransform {
        /* WHY: Clamp to valid byte boundaries. */
        let lo = sel_start.min(sel_end).min(buffer.len());
        let hi = sel_start.max(sel_end).min(buffer.len());
        let selected = &buffer[lo..hi];

        match op {
            /* WHY: Inline wrapping operations */
            MarkdownAuthoringOp::Bold => {
                AuthoringUtils::wrap_inline(buffer, lo, hi, selected, "**", "**")
            }
            MarkdownAuthoringOp::Italic => {
                AuthoringUtils::wrap_inline(buffer, lo, hi, selected, "*", "*")
            }
            MarkdownAuthoringOp::Strikethrough => {
                AuthoringUtils::wrap_inline(buffer, lo, hi, selected, "~~", "~~")
            }
            MarkdownAuthoringOp::InlineCode => {
                AuthoringUtils::wrap_inline(buffer, lo, hi, selected, "`", "`")
            }
            /* WHY: Block-level heading / prefix operations */
            MarkdownAuthoringOp::Heading1 => {
                AuthoringUtils::prefix_line(buffer, lo, hi, selected, "# ")
            }
            MarkdownAuthoringOp::Heading2 => {
                AuthoringUtils::prefix_line(buffer, lo, hi, selected, "## ")
            }
            MarkdownAuthoringOp::Heading3 => {
                AuthoringUtils::prefix_line(buffer, lo, hi, selected, "### ")
            }
            MarkdownAuthoringOp::BulletList => {
                AuthoringUtils::prefix_each_line(buffer, lo, hi, selected, "- ")
            }
            MarkdownAuthoringOp::NumberedList => {
                AuthoringUtils::prefix_each_line_numbered(buffer, lo, hi, selected)
            }
            MarkdownAuthoringOp::Blockquote => {
                AuthoringUtils::prefix_each_line(buffer, lo, hi, selected, "> ")
            }
            MarkdownAuthoringOp::CodeBlock => {
                AuthoringUtils::wrap_block(buffer, lo, hi, selected, "```\n", "\n```")
            }
            MarkdownAuthoringOp::HorizontalRule => {
                AuthoringUtils::insert_snippet(buffer, lo, "\n---\n")
            }
            MarkdownAuthoringOp::InsertLink => {
                if selected.is_empty() {
                    AuthoringUtils::insert_snippet(buffer, lo, "[link text](url)")
                } else {
                    AuthoringUtils::wrap_inline(buffer, lo, hi, selected, "[", "](url)")
                }
            }
            MarkdownAuthoringOp::InsertTable => AuthoringUtils::insert_snippet(
                buffer,
                lo,
                "| Header 1 | Header 2 | Header 3 |\n| -------- | -------- | -------- |\n| Cell 1   | Cell 2   | Cell 3   |\n",
            ),
        }
    }
}

#[cfg(test)]
include!("authoring_tests.rs");
