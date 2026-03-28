mod comment_style;
mod lazy_code;
mod magic_numbers;
mod performance;
mod prohibited_attrs;
mod prohibited_types;

pub use comment_style::lint_comment_style;
pub use lazy_code::lint_lazy_code;
pub use magic_numbers::lint_magic_numbers;
pub use performance::lint_performance;
pub use prohibited_attrs::lint_prohibited_attributes;
pub use prohibited_types::lint_prohibited_types;
