use crate::Violation;
use std::path::Path;

pub struct CommentStyleOps;

impl CommentStyleOps {
    pub fn lint(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
        let mut violations = Vec::new();
        let Ok(content) = std::fs::read_to_string(path) else {
            return violations;
        };

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//")
                && !trimmed.starts_with("///")
                && !trimmed.starts_with("//!")
            {
                let text = trimmed.trim_start_matches('/').trim();
                if !text.is_empty()
                    && !text.starts_with("WHY:")
                    && !text.starts_with("SAFETY:")
                    && !text.starts_with("TODO:")
                {
                    // WHY: Broad check catches most non-doc comments that lack context.
                    violations.push(Violation {
                         file: path.to_path_buf(),
                         line: line_idx + 1,
                         column: 1,
                         message: "Comments must start with `// WHY:` or `// SAFETY:`. Code should be self-documenting.".to_string(),
                     });
                }
            }
        }

        violations
    }
}
