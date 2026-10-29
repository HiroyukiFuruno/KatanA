#[cfg(target_os = "macos")]
pub(super) fn relieve() -> usize {
    use std::ffi::c_void;
    unsafe extern "C" {
        fn malloc_zone_pressure_relief(zone: *mut c_void, goal: usize) -> usize;
    }
    /* SAFETY: NULLで登録済みzone全体を対象にし、goal 0で最大限の返却候補を要求する。
     * ポインタを保持せず、呼出し中だけlibmallocの契約に従う。 */
    unsafe { malloc_zone_pressure_relief(std::ptr::null_mut(), 0) }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn relieve() -> usize {
    0
}
