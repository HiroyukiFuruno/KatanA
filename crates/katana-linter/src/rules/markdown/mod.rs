/* WHY: Re-export everything from the external katana-markdown-linter crate.
This replaces the internal implementation with the dedicated library. */

pub use katana_markdown_linter::rules::markdown::*;
