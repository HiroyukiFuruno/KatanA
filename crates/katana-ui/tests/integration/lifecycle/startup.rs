use crate::integration::harness_utils::setup_harness;
use egui_kittest::kittest::Queryable;

#[test]
fn test_integration_application_startup() {
    /* WHY: Verify that the application starts correctly and shows the default empty workspace state. */
    let mut harness = setup_harness();
    harness.step();
    let _node = harness.get_by_label("No workspace open.");
}
