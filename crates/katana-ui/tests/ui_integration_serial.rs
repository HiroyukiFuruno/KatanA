#![allow(clippy::module_inception)]
#![allow(deprecated)]

pub static SERIAL_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[path = "integration/diagram_rendering.rs"]
mod diagram_rendering;
#[path = "integration/integration_i18n.rs"]
mod integration_i18n;
