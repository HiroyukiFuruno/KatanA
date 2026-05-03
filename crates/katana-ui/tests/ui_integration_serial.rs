#![allow(clippy::module_inception)]
#![allow(deprecated)]

use std::sync::MutexGuard;

pub static SERIAL_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn get_serial_test_mutex() -> &'static std::sync::Mutex<()> {
    &SERIAL_TEST_MUTEX
}

pub fn lock_serial_test_mutex() -> MutexGuard<'static, ()> {
    get_serial_test_mutex()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub mod integration {
    pub use super::get_serial_test_mutex;
    pub use super::lock_serial_test_mutex;
    #[path = "harness_utils.rs"]
    pub mod harness_utils;
    #[path = "test_helpers.rs"]
    pub mod test_helpers;
}

#[path = "integration/preview_pane/diagrams.rs"]
pub mod diagram_rendering;
#[path = "integration/preview_pane/rendering.rs"]
pub mod html_block_tests;
#[path = "integration/settings/integration_i18n.rs"]
pub mod integration_i18n;
#[path = "integration/tree_layout.rs"]
mod tree_layout;
