/* WHY: Refactored slideshow entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

mod controls;
mod modal;
mod settings;

pub use controls::SlideshowControlsOps;
pub use modal::SlideshowModalOps;
pub use settings::SlideshowSettingsOps;

/* WHY: Commented out missing test module references to unblock compilation/linting. */
/* mod slideshow_tests; */
