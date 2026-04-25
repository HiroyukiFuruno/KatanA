pub mod adapter;
pub mod image;
pub mod section;
pub mod types;

pub use adapter::*;
pub use section::*;
pub use types::*;

#[cfg(test)]
mod tests;

impl PreviewFlattenOps {
    pub fn flatten_list_code_blocks(source: &str) -> String {
        let mut result = String::with_capacity(source.len());
        let mut in_indented_fence = false;
        let mut fence_indent = 0;

        for line in source.lines() {
            if in_indented_fence {
                let stripped = strip_leading_spaces(line, fence_indent);
                if stripped.trim_start().starts_with("```") {
                    result.push_str(stripped.trim_start());
                    in_indented_fence = false;
                } else {
                    result.push_str(stripped);
                }
            } else {
                let indent = count_leading_spaces(line);
                if indent >= 2 && line.trim_start().starts_with("```") {
                    in_indented_fence = true;
                    fence_indent = indent;
                    result.push_str(line.trim_start());
                } else {
                    result.push_str(line);
                }
            }
            result.push('\n');
        }

        if !source.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }
        result
    }
}

fn count_leading_spaces(s: &str) -> usize {
    s.bytes().take_while(|b| *b == b' ').count()
}

fn strip_leading_spaces(s: &str, max: usize) -> &str {
    let n = count_leading_spaces(s).min(max);
    &s[n..]
}
