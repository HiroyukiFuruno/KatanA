/* WHY: Section: Rule submodule aggregator
=======================================================
  All markdownlint rule implementations live under this directory.
  Each file is capped at 200 lines per coding-rules §2.1. */

pub mod blockquote;
pub use blockquote::*;

pub mod content;
pub use content::*;

pub mod content_ext;
pub use content_ext::*;

pub mod heading;
pub use heading::*;

pub mod heading_ext;
pub use heading_ext::*;

pub mod heading_style;
pub use heading_style::*;

pub mod image;
pub use image::*;

pub mod list;
pub use list::*;

pub mod list_ext;
pub use list_ext::*;

pub mod spaces_in_code;
pub use spaces_in_code::*;

pub mod spaces_in_emphasis;
pub use spaces_in_emphasis::*;

pub mod style;
pub use style::*;

pub mod table;
pub use table::*;

pub mod whitespace;
pub use whitespace::*;
