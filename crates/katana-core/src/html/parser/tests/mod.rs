/* WHY: Root module for HTML parser tests. Organizes tests by category into sub-modules for better maintainability and to comply with file length limits. */

use super::*;
// use std::path::PathBuf;

mod basic_tags;
mod complex_structures;
mod edge_cases;
mod markdown_inlines;

fn parser() -> HtmlParser<'static> {
    HtmlParser::new(Path::new("/project"))
}
