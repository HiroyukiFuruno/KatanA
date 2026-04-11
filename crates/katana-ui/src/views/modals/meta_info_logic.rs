use katana_core::document::Document;

pub struct MetaInfoLogic;

impl MetaInfoLogic {
    /* WHY: Human-readable size: bytes / KB / MB / GB */
    pub fn format_file_size(bytes: u64) -> String {
        const KB: u64 = 1_024;
        const MB: u64 = KB * 1_024;
        const GB: u64 = MB * 1_024;
        match bytes {
            b if b >= GB => format!("{:.2} GB ({} bytes)", b as f64 / GB as f64, b),
            b if b >= MB => format!("{:.2} MB ({} bytes)", b as f64 / MB as f64, b),
            b if b >= KB => format!("{:.1} KB ({} bytes)", b as f64 / KB as f64, b),
            b => format!("{} bytes", b),
        }
    }

    /* WHY: Format SystemTime as local datetime string using chrono. */
    pub fn format_system_time(t: std::time::SystemTime) -> String {
        use chrono::{DateTime, Local};
        let dt: DateTime<Local> = t.into();
        dt.format("%Y-%m-%d  %H:%M:%S").to_string()
    }

    #[cfg(unix)]
    /* WHY: Resolve Unix UID to username via getpwuid_r. */
    pub fn resolve_unix_owner(uid: u32) -> String {
        use std::ffi::CStr;
        /* WHY: Buffer size for getpwuid_r. */
        const PWD_BUF_SIZE: usize = 512;

        let mut buf = vec![0i8; PWD_BUF_SIZE];
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result: *mut libc::passwd = std::ptr::null_mut();
        let ret = unsafe {
            libc::getpwuid_r(
                uid,
                pwd.as_mut_ptr(),
                buf.as_mut_ptr(),
                buf.len(),
                &mut result,
            )
        };

        let Some(r) = (if ret == 0 {
            unsafe { result.as_ref() }
        } else {
            None
        }) else {
            return uid.to_string();
        };

        if r.pw_name.is_null() {
            return uid.to_string();
        }

        let Ok(name) = unsafe { CStr::from_ptr(r.pw_name) }.to_str() else {
            return uid.to_string();
        };

        format!("{} ({})", name, uid)
    }

    #[cfg(unix)]
    /* WHY: Format Unix file mode as "drwxr-xr-x" style string. */
    pub fn format_unix_permissions(mode: u32) -> String {
        /* WHY: Unix mode masks and bits for permission formatting. */
        const UNIX_MODE_MASK: u32 = 0o170000;
        const UNIX_MODE_DIR: u32 = 0o040000;
        const UNIX_MODE_LINK: u32 = 0o120000;
        const MODE_BITS: [(u32, char); 9] = [
            (0o400, 'r'),
            (0o200, 'w'),
            (0o100, 'x'),
            (0o040, 'r'),
            (0o020, 'w'),
            (0o010, 'x'),
            (0o004, 'r'),
            (0o002, 'w'),
            (0o001, 'x'),
        ];

        let file_type = match mode & UNIX_MODE_MASK {
            UNIX_MODE_DIR => 'd',
            UNIX_MODE_LINK => 'l',
            _ => '-',
        };
        let perm: String = MODE_BITS
            .iter()
            .map(|&(bit, c)| if mode & bit != 0 { c } else { '-' })
            .collect();
        format!("{}{}", file_type, perm)
    }
}
