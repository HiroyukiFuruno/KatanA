use super::resize_events::{ResizeHistory, ResizeIdentity};
use std::cell::Cell;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Shell::{
    DefSubclassProc, GetWindowSubclass, RemoveWindowSubclass, SetWindowSubclass,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClientRect, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_NCDESTROY, WM_SETTINGCHANGE, WM_SIZE,
    WM_STYLECHANGED, WM_THEMECHANGED,
};

pub(super) mod creation;

const SUBCLASS_ID: usize = 0x4b41_5253;
const DIMENSION_MASK: u32 = u16::MAX as u32;

struct ResizeState {
    history: Cell<ResizeHistory>,
    forwarded: Cell<u64>,
    suppressed: Cell<u64>,
    record: bool,
}

#[cfg(not(test))]
pub(super) struct WindowsResizeOps;

#[cfg(not(test))]
impl WindowsResizeOps {
    pub(super) fn install(context: &eframe::CreationContext<'_>) -> std::io::Result<()> {
        use eframe::wgpu::rwh::{HasWindowHandle, RawWindowHandle};
        let handle = context.window_handle().map_err(std::io::Error::other)?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return Err(std::io::Error::other(
                "The native window has no Win32 handle",
            ));
        };
        ensure_registered(handle.hwnd.get() as HWND)
    }
}

fn ensure_registered(window: HWND) -> std::io::Result<()> {
    let mut existing = 0;
    /* WHY: 生成時に登録した履歴を保持し、後段で同じ状態を置き換えない。 */
    if unsafe { GetWindowSubclass(window, Some(window_callback), SUBCLASS_ID, &mut existing) } != 0
        && existing != 0
    {
        return Ok(());
    }
    register(window)
}

fn register(window: HWND) -> std::io::Result<()> {
    let mut existing = 0;
    /* WHY: 同じUI threadの所有windowへ一度だけ登録し、既存状態の上書きによる漏れを防ぐ。 */
    if unsafe { GetWindowSubclass(window, Some(window_callback), SUBCLASS_ID, &mut existing) } != 0
    {
        return Err(std::io::Error::other(
            "The native resize filter is already installed",
        ));
    }
    let state = Box::new(ResizeState {
        history: Cell::new(ResizeHistory::default()),
        forwarded: Cell::new(0),
        suppressed: Cell::new(0),
        record: std::env::var_os("KATANA_STARTUP_HEARTBEAT").is_some(),
    });
    let pointer = Box::into_raw(state);
    /* WHY: 登録成功後の状態はWM_NCDESTROYまでwindow自身が所有する。 */
    if unsafe { SetWindowSubclass(window, Some(window_callback), SUBCLASS_ID, pointer as usize) }
        == 0
    {
        unsafe { drop(Box::from_raw(pointer)) };
        return Err(std::io::Error::other(
            "Windows rejected the native resize filter",
        ));
    }
    Ok(())
}

unsafe extern "system" fn window_callback(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    if message == WM_NCDESTROY {
        /* WHY: 下位callbackへの委譲中は状態を保持し、最終通知の復帰後に一度だけ解放する。 */
        unsafe { RemoveWindowSubclass(window, Some(window_callback), id) };
        let result = unsafe { DefSubclassProc(window, message, wparam, lparam) };
        unsafe { drop(Box::from_raw(data as *mut ResizeState)) };
        return result;
    }
    if observe(window, message, wparam, lparam, data) {
        return 0;
    }
    /* WHY: 状態の借用を委譲前に終え、同期的な再入通知と可変参照を重ねない。 */
    unsafe { DefSubclassProc(window, message, wparam, lparam) }
}

fn observe(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM, data: usize) -> bool {
    /* WHY: このpointerは同じUI threadで登録し、WM_NCDESTROYの復帰まで生存する。 */
    let state = unsafe { &*(data as *const ResizeState) };
    let mut history = state.history.get();
    if matches!(
        message,
        WM_DISPLAYCHANGE | WM_DPICHANGED | WM_SETTINGCHANGE | WM_STYLECHANGED | WM_THEMECHANGED
    ) {
        history.invalidate();
        state.history.set(history);
        if state.record {
            tracing::info!(event = "native_resize_filter_invalidate", message);
        }
        return false;
    }
    if message != WM_SIZE {
        return false;
    }
    let identity = native_identity(window, wparam, lparam);
    let suppressed = history.suppress(identity);
    state.history.set(history);
    record_observation(state, identity, suppressed, wparam, lparam);
    suppressed
}

fn native_identity(window: HWND, kind: WPARAM, packed_size: LPARAM) -> Option<ResizeIdentity> {
    let packed = packed_size as u32;
    let width = packed & DIMENSION_MASK;
    let height = (packed >> u16::BITS) & DIMENSION_MASK;
    let mut rect = RECT::default();
    /* WHY: 古いサイズ通知や16bitを超える寸法を同一イベントと誤判定しない。 */
    if unsafe { GetClientRect(window, &mut rect) } == 0
        || u32::try_from(rect.right.checked_sub(rect.left)?).ok()? != width
        || u32::try_from(rect.bottom.checked_sub(rect.top)?).ok()? != height
    {
        return None;
    }
    Some(ResizeIdentity {
        kind,
        width,
        height,
        dpi: unsafe { GetDpiForWindow(window) },
        monitor: unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) } as isize,
    })
}

fn record_observation(
    state: &ResizeState,
    identity: Option<ResizeIdentity>,
    suppressed: bool,
    kind: WPARAM,
    packed_size: LPARAM,
) {
    let count = if suppressed {
        &state.suppressed
    } else {
        &state.forwarded
    };
    count.set(count.get().saturating_add(1));
    if state.record {
        tracing::info!(
            event = "native_resize_filter",
            ?identity,
            suppressed,
            kind,
            packed_size,
            forwarded_count = state.forwarded.get(),
            suppressed_count = state.suppressed.get()
        );
    }
}

#[cfg(test)]
mod tests;
