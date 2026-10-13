use super::clipboard_image::{ClipboardImageOps, ClipboardImagePayload};
use std::path::Path;

impl ClipboardImageOps {
    pub(super) fn macos_pasteboard_has_image() -> bool {
        let Ok(output) = katana_core::system::ProcessService::create_command("osascript")
            .arg("-e")
            .arg("clipboard info")
            .output()
        else {
            return false;
        };
        if !output.status.success() {
            return false;
        }
        let info = String::from_utf8_lossy(&output.stdout);
        Self::macos_clipboard_info_references_supported_image(&info)
    }

    pub(super) fn macos_clipboard_info_references_supported_image(info: &str) -> bool {
        [
            "«class PNGf»",
            "JPEG picture",
            "TIFF picture",
            "«class TIFF»",
        ]
        .iter()
        .any(|image_type| info.contains(image_type))
    }

    pub(super) fn read_macos_pasteboard_image() -> Result<ClipboardImagePayload, String> {
        let mut errors = Vec::new();

        match Self::read_macos_native_pasteboard_image() {
            Ok(payload) => return Ok(payload),
            Err(err) => errors.push(err),
        }

        for (apple_type, extension) in [
            ("«class PNGf»", "png"),
            ("JPEG picture", "jpg"),
            ("TIFF picture", "tiff"),
            ("«class TIFF»", "tiff"),
        ] {
            let payload = Self::read_macos_pasteboard_type(apple_type)
                .and_then(|bytes| Self::normalize_macos_pasteboard_bytes(bytes, extension));
            if payload.is_ok() {
                return payload;
            }
            if let Err(err) = payload {
                errors.push(err);
            }
        }
        errors.push("macOS pasteboard image formats were not available".to_string());
        Err(errors.join("; "))
    }

    fn read_macos_native_pasteboard_image() -> Result<ClipboardImagePayload, String> {
        let mut bytes_ptr = std::ptr::null_mut();
        let mut bytes_len: std::ffi::c_ulong = 0;
        let ok = unsafe {
            macos_clipboard_ffi::katana_read_clipboard_image_png(&mut bytes_ptr, &mut bytes_len)
        };
        if !ok || bytes_ptr.is_null() || bytes_len == 0 {
            return Err("macOS native pasteboard image data was not available".to_string());
        }

        let bytes =
            unsafe { std::slice::from_raw_parts(bytes_ptr.cast_const(), bytes_len as usize) }
                .to_vec();
        unsafe {
            macos_clipboard_ffi::katana_free_clipboard_image(bytes_ptr);
        }
        Ok(ClipboardImagePayload {
            bytes,
            extension: "png",
        })
    }

    fn read_macos_pasteboard_type(apple_type: &str) -> Result<Vec<u8>, String> {
        let path =
            std::env::temp_dir().join(format!("katana_clipboard_image_{}", uuid::Uuid::new_v4()));
        let path_arg = Self::apple_script_string(&path);
        let status = katana_core::system::ProcessService::create_command("osascript")
            .arg("-e")
            .arg(format!("set outPath to \"{path_arg}\""))
            .arg("-e")
            .arg(format!("set imageData to the clipboard as {apple_type}"))
            .arg("-e")
            .arg("set outFile to open for access POSIX file outPath with write permission")
            .arg("-e")
            .arg("set eof of outFile to 0")
            .arg("-e")
            .arg("write imageData to outFile")
            .arg("-e")
            .arg("close access outFile")
            .status()
            .map_err(|err| err.to_string())?;
        if !status.success() {
            let _ = std::fs::remove_file(&path);
            return Err(format!("osascript failed for {apple_type}"));
        }
        let bytes = std::fs::read(&path).map_err(|err| err.to_string())?;
        let _ = std::fs::remove_file(path);
        Ok(bytes)
    }

    fn normalize_macos_pasteboard_bytes(
        bytes: Vec<u8>,
        extension: &'static str,
    ) -> Result<ClipboardImagePayload, String> {
        if extension != "tiff" {
            return Ok(ClipboardImagePayload { bytes, extension });
        }

        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Tiff)
            .map_err(|err| err.to_string())?
            .into_rgba8();
        let (width, height) = image.dimensions();
        let bytes = Self::rgba_to_png_bytes(width as usize, height as usize, &image.into_raw())?;
        Ok(ClipboardImagePayload {
            bytes,
            extension: "png",
        })
    }

    fn apple_script_string(path: &Path) -> String {
        path.to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
    }
}

mod macos_clipboard_ffi {
    unsafe extern "C" {
        pub(super) fn katana_read_clipboard_image_png(
            out_bytes: *mut *mut u8,
            out_len: *mut std::ffi::c_ulong,
        ) -> bool;
        pub(super) fn katana_free_clipboard_image(bytes: *mut u8);
    }
}
