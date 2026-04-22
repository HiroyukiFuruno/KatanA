/* WHY: Section: Rule submodule aggregator
=======================================================
  All markdownlint rule implementations live under this directory.
  Each file is capped at 200 lines per coding-rules §2.1. */

pub mod blockquote;
pub use blockquote::*;

pub mod content;
pub use content::*;

pub mod heading;
pub use heading::*;

pub mod heading_ext;
pub use heading_ext::*;

pub mod list;
pub use list::*;

pub mod style;
pub use style::*;

pub mod whitespace;
pub use whitespace::*;
