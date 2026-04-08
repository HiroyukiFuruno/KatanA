/* WHY: Platform Contract definitions and facade.
Provides consistent cross-platform constants and capabilities queries.
Useful for providing correct UI cues (like modifier keys) and determining
update/install capabilities without hardcoding `cfg(target_os)` throughout the app. */

/* WHY: Defines the platform contract operations. */
pub struct PlatformContractOps;

impl PlatformContractOps {
    /* WHY: Defines the primary modifier key identifier used by `egui` to ensure shortcuts
    render and behave correctly on standard platforms. */
    #[cfg(target_os = "macos")]
    pub const PRIMARY_MODIFIER_NAME: &'static str = "Cmd";
    #[cfg(not(target_os = "macos"))]
    pub const PRIMARY_MODIFIER_NAME: &'static str = "Ctrl";
    /* WHY: Returns whether the current OS provides a native global menu (e.g., macOS). */
    #[must_use]
    pub fn has_native_global_menu() -> bool {
        #[cfg(target_os = "macos")]
        {
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /* WHY: Returns the OS-specific instruction or behavior for application updates. */
    #[must_use]
    pub fn update_install_mode() -> &'static str {
        #[cfg(target_os = "macos")]
        {
            "dmg"
        }
        #[cfg(target_os = "windows")]
        {
            "zip_extract"
        }
        #[cfg(target_os = "linux")]
        {
            "tar_extract"
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            "unknown"
        }
    }
}
