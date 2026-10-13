#![cfg(target_os = "macos")]

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
const OPTION: c_ulong = 1 << 19;
const TAG_PASTE_CLIPBOARD_IMAGE: i32 = 37;

fn should_intercept(characters: &str, flags: c_ulong) -> bool {
    let characters = CString::new(characters).unwrap();
    unsafe {
        katana_should_intercept_clipboard_image_paste_for_test(
            true,
            true,
            flags,
            characters.as_ptr(),
        )
    }
}

#[test]
fn native_clipboard_image_shortcut_accepts_plain_command_v_only() {
    assert!(should_intercept("v", COMMAND));
    assert!(should_intercept("V", COMMAND));
    assert!(!should_intercept("b", COMMAND));
    assert!(!should_intercept("v", 0));
    assert!(!should_intercept("v", COMMAND | OPTION));
}

#[test]
fn native_clipboard_image_shortcut_returns_image_paste_action() {
    let characters = CString::new("v").unwrap();
    let action = unsafe {
        katana_clipboard_image_paste_action_for_test(true, true, COMMAND, characters.as_ptr())
    };
    assert_eq!(action, TAG_PASTE_CLIPBOARD_IMAGE);
}
