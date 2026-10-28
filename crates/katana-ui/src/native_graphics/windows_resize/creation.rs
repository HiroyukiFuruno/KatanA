use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::rc::Rc;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CWPSTRUCT, CallNextHookEx, GetParent, GetWindowTextW, GetWindowThreadProcessId, HC_ACTION,
    HHOOK, SetWindowsHookExW, UnhookWindowsHookEx, WH_CALLWNDPROC, WM_CREATE,
};

const WINDOW_TITLE: &str = "KatanA";
const TITLE_CAPACITY: usize = 32;

thread_local! {
    static REGISTRATION: RefCell<Option<Rc<Registration>>> = const { RefCell::new(None) };
}

struct Registration {
    handle: Cell<HHOOK>,
}

pub(crate) struct CreationHook {
    registration: Rc<Registration>,
}

impl CreationHook {
    pub(in crate::native_graphics) fn install() -> std::io::Result<Self> {
        if REGISTRATION.with(|slot| slot.borrow().is_some()) {
            return Err(std::io::Error::other(
                "The creation hook is already installed",
            ));
        }
        /* WHY: HWND生成後のApp生成では、既に待機列へ入ったサイズ通知に間に合わない。 */
        let handle = unsafe {
            SetWindowsHookExW(
                WH_CALLWNDPROC,
                Some(on_creation),
                null_mut(),
                GetCurrentThreadId(),
            )
        };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        let registration = Rc::new(Registration {
            handle: Cell::new(handle),
        });
        REGISTRATION.with(|slot| slot.replace(Some(Rc::clone(&registration))));
        Ok(Self { registration })
    }
}

impl Drop for CreationHook {
    fn drop(&mut self) {
        release(&self.registration);
    }
}

fn release(registration: &Rc<Registration>) {
    REGISTRATION.with(|slot| {
        let mut current = slot.borrow_mut();
        if current
            .as_ref()
            .is_some_and(|value| Rc::ptr_eq(value, registration))
        {
            current.take();
        }
    });
    let handle = registration.handle.replace(null_mut());
    if !handle.is_null() && unsafe { UnhookWindowsHookEx(handle) } == 0 {
        tracing::error!(event = "native_resize_creation_hook_remove_failed", error = %std::io::Error::last_os_error());
    }
}

unsafe extern "system" fn on_creation(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && lparam != 0 {
        let message = unsafe { &*(lparam as *const CWPSTRUCT) };
        if message.message == WM_CREATE && is_owned_root(message.hwnd) {
            register_created_root(message.hwnd);
        }
    }
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

fn is_owned_root(window: HWND) -> bool {
    if unsafe { GetWindowThreadProcessId(window, null_mut()) } != unsafe { GetCurrentThreadId() }
        || !unsafe { GetParent(window) }.is_null()
    {
        return false;
    }
    let mut title = [0; TITLE_CAPACITY];
    let length = unsafe { GetWindowTextW(window, title.as_mut_ptr(), TITLE_CAPACITY as i32) };
    usize::try_from(length)
        .ok()
        .and_then(|length| title.get(..length))
        .is_some_and(|title| title.iter().copied().eq(WINDOW_TITLE.encode_utf16()))
}

fn register_created_root(window: HWND) {
    let registration = REGISTRATION.with(|slot| slot.borrow().as_ref().map(Rc::clone));
    let Some(registration) = registration else {
        return;
    };
    /* WHY: 同じUI threadの最初の所有rootだけへ適用し、他windowの生成を監視し続けない。 */
    match super::register(window) {
        Ok(()) => tracing::info!(event = "native_resize_filter_installed_at_creation"),
        Err(error) => tracing::error!(event = "native_resize_creation_install_failed", %error),
    }
    release(&registration);
}

#[cfg(test)]
mod tests;
