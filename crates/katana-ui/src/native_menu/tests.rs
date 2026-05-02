use std::ffi::{CString, c_char, c_ulong};

unsafe extern "C" {
    fn katana_should_intercept_clipboard_image_paste_for_test(
        editor_focused: bool,
        image_available: bool,
        modifier_flags: c_ulong,
        characters: *const c_char,
    ) -> bool;
    fn katana_clipboard_image_paste_action_for_test(
        editor_focused: bool,
        image_available: bool,
        modifier_flags: c_ulong,
        characters: *const c_char,
    ) -> i32;
}

const COMMAND: c_ulong = 1 << 20;
const SHIFT: c_ulong = 1 << 17;

fn should_intercept(editor_focused: bool, image_available: bool, flags: c_ulong) -> bool {
    let characters = CString::new("v").unwrap();
    unsafe {
        katana_should_intercept_clipboard_image_paste_for_test(
            editor_focused,
            image_available,
            flags,
            characters.as_ptr(),
        )
    }
}

#[test]
fn native_clipboard_image_shortcut_requires_editor_focus_and_image() {
    assert!(should_intercept(true, true, COMMAND));
    assert!(!should_intercept(false, true, COMMAND));
    assert!(!should_intercept(true, false, COMMAND));
}

#[test]
fn native_clipboard_image_shortcut_does_not_steal_shift_paste() {
    assert!(!should_intercept(true, true, COMMAND | SHIFT));
}

#[test]
fn native_clipboard_image_shortcut_returns_ingest_action_tag() {
    let characters = CString::new("v").unwrap();
    let action = unsafe {
        katana_clipboard_image_paste_action_for_test(true, true, COMMAND, characters.as_ptr())
    };
    assert_eq!(action, super::ffi::TAG_PASTE_CLIPBOARD_IMAGE);
}
