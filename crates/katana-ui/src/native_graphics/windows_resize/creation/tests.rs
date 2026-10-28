use super::{CreationHook, REGISTRATION};
use crate::native_graphics::windows_resize::tests::{
    NativeWindow, counts, registered_state, send_size,
};
use crate::native_graphics::windows_resize::{SUBCLASS_ID, ensure_registered, window_callback};
use std::ptr::null_mut;
use windows_sys::Win32::UI::Shell::GetWindowSubclass;

fn is_registered(window: &NativeWindow) -> bool {
    let mut state = 0;
    unsafe { GetWindowSubclass(window.0, Some(window_callback), SUBCLASS_ID, &mut state) != 0 }
}

#[test]
fn creation_hook_registers_owned_window_before_late_app_installation() {
    let _hook = CreationHook::install().expect("install on the current UI thread");
    let window = NativeWindow::create_with_title("KatanA", null_mut());
    let original_state = registered_state(window.0);
    assert!(REGISTRATION.with(|slot| slot.borrow().is_none()));
    let before = counts(window.0);
    send_size(window.0, 0);
    send_size(window.0, 0);
    let after = counts(window.0);
    assert_eq!(after.0 + after.1, before.0 + before.1 + 2);
    assert!(after.1 > before.1);
    ensure_registered(window.0).expect("adopt the existing creation-time state");
    assert_eq!(registered_state(window.0), original_state);
    send_size(window.0, 0);
    assert_eq!(counts(window.0), (after.0, after.1 + 1));
}

#[test]
fn creation_hook_ignores_unrelated_roots_and_children_without_consuming_registration() {
    let _hook = CreationHook::install().expect("install on the current UI thread");
    assert!(CreationHook::install().is_err());
    let unrelated = NativeWindow::create_with_title("unrelated", null_mut());
    let child = NativeWindow::create_with_title("KatanA", unrelated.0);
    assert!(!is_registered(&unrelated));
    assert!(!is_registered(&child));
    assert!(REGISTRATION.with(|slot| slot.borrow().is_some()));
    let owned = NativeWindow::create_with_title("KatanA", null_mut());
    assert!(is_registered(&owned));
    assert!(REGISTRATION.with(|slot| slot.borrow().is_none()));
}

#[test]
fn dropping_unused_creation_hook_leaves_later_windows_untouched() {
    let hook = CreationHook::install().expect("install on the current UI thread");
    drop(hook);
    assert!(REGISTRATION.with(|slot| slot.borrow().is_none()));
    let window = NativeWindow::create_with_title("KatanA", null_mut());
    assert!(!is_registered(&window));
}

#[test]
fn completed_creation_guard_does_not_release_a_new_registration() {
    let first_hook = CreationHook::install().expect("install first hook");
    let first_window = NativeWindow::create_with_title("KatanA", null_mut());
    assert!(is_registered(&first_window));
    let _second_hook = CreationHook::install().expect("install next hook after completion");
    drop(first_hook);
    assert!(REGISTRATION.with(|slot| slot.borrow().is_some()));
    let second_window = NativeWindow::create_with_title("KatanA", null_mut());
    assert!(is_registered(&second_window));
}
