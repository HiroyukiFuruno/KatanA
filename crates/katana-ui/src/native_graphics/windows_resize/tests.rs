use super::{ResizeState, SUBCLASS_ID, observe, register, window_callback};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::UI::Shell::GetWindowSubclass;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, GetClientRect, IsWindow, SendMessageW, WM_DISPLAYCHANGE,
    WM_DPICHANGED, WM_SETTINGCHANGE, WM_SIZE, WM_STYLECHANGED, WM_THEMECHANGED, WS_CHILD,
    WS_OVERLAPPEDWINDOW,
};

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

pub(super) struct NativeWindow(pub(super) HWND);

impl NativeWindow {
    fn create() -> Self {
        Self::create_with_title("KatanA resize regression", null_mut())
    }

    pub(super) fn create_with_title(title: &str, parent: HWND) -> Self {
        let class: Vec<u16> = "STATIC".encode_utf16().chain(Some(0)).collect();
        let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
        let style = if parent.is_null() {
            WS_OVERLAPPEDWINDOW
        } else {
            WS_CHILD
        };
        let handle = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                style,
                0,
                0,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                parent,
                null_mut(),
                null_mut(),
                null(),
            )
        };
        assert!(!handle.is_null());
        Self(handle)
    }
}

impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.0) };
    }
}

pub(super) fn counts(window: HWND) -> (u64, u64) {
    let pointer = registered_state(window);
    let state = unsafe { &*(pointer as *const ResizeState) };
    (state.forwarded.get(), state.suppressed.get())
}

pub(super) fn registered_state(window: HWND) -> usize {
    let mut pointer = 0;
    assert_ne!(
        unsafe { GetWindowSubclass(window, Some(window_callback), SUBCLASS_ID, &mut pointer) },
        0
    );
    pointer
}

pub(super) fn send_size(window: HWND, kind: usize) {
    let mut rect = RECT::default();
    assert_ne!(unsafe { GetClientRect(window, &mut rect) }, 0);
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    assert!((1..65536).contains(&width));
    assert!((1..65536).contains(&height));
    let packed = (width as isize) | ((height as isize) << u16::BITS);
    unsafe { SendMessageW(window, WM_SIZE, kind, packed) };
}

#[test]
fn real_window_forwards_first_size_and_state_changes_but_suppresses_duplicates() {
    let window = NativeWindow::create();
    register(window.0).expect("install on owned real Win32 window");
    send_size(window.0, 0);
    send_size(window.0, 0);
    assert_eq!(counts(window.0), (1, 1));
    send_size(window.0, 2);
    assert_eq!(counts(window.0), (2, 1));
    send_size(window.0, 0);
    assert_eq!(counts(window.0), (3, 1));
}

#[test]
fn real_window_preserves_registration_and_invalidates_on_display_change() {
    let window = NativeWindow::create();
    register(window.0).expect("install on owned real Win32 window");
    assert!(register(window.0).is_err());
    send_size(window.0, 0);
    send_size(window.0, 0);
    unsafe { SendMessageW(window.0, WM_DISPLAYCHANGE, 0, 0) };
    send_size(window.0, 0);
    assert_eq!(counts(window.0), (2, 1));
    let handle = window.0;
    drop(window);
    assert_eq!(unsafe { IsWindow(handle) }, 0);
}

#[test]
fn registered_observer_preserves_each_configuration_invalidation() {
    let window = NativeWindow::create();
    register(window.0).expect("install on owned real Win32 window");
    for message in [
        WM_DPICHANGED,
        WM_DISPLAYCHANGE,
        WM_SETTINGCHANGE,
        WM_STYLECHANGED,
        WM_THEMECHANGED,
    ] {
        send_size(window.0, 0);
        send_size(window.0, 0);
        let before = counts(window.0);
        assert!(!observe(
            window.0,
            message,
            0,
            0,
            registered_state(window.0)
        ));
        send_size(window.0, 0);
        assert_eq!(counts(window.0).0, before.0 + 1);
    }
}
