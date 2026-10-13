use image::ImageEncoder;
use std::path::Path;

const RGBA_CHANNEL_COUNT: usize = 4;

pub(crate) struct ClipboardImagePayload {
    pub bytes: Vec<u8>,
    pub extension: &'static str,
}

pub(crate) struct ClipboardImageOps;

impl ClipboardImageOps {
    pub(crate) fn has_image_payload() -> bool {
        if Self::read_arboard_image().is_ok() {
            return true;
        }
        if Self::read_arboard_file_image().is_ok() {
            return true;
        }
        if super::clipboard_file_url::ClipboardFileUrlOps::read_image_payload().is_ok() {
            return true;
        }
        #[cfg(target_os = "macos")]
        {
            Self::read_macos_pasteboard_image().is_ok()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    pub(crate) fn read_image_payload() -> Result<ClipboardImagePayload, String> {
        let mut errors = Vec::new();
        let image_error = match Self::read_arboard_image() {
            Ok(payload) => return Ok(payload),
            Err(err) => err,
        };
        errors.push(image_error);

        match Self::read_arboard_file_image() {
            Ok(payload) => return Ok(payload),
            Err(err) => errors.push(err),
        }

        match super::clipboard_file_url::ClipboardFileUrlOps::read_image_payload() {
            Ok(payload) => return Ok(payload),
            Err(err) => errors.push(err),
        }

        #[cfg(target_os = "macos")]
        match Self::read_macos_pasteboard_image() {
            Ok(payload) => return Ok(payload),
            Err(err) => errors.push(err),
        }

        Err(format!(
            "No clipboard image data was available: {}",
            errors.join("; ")
        ))
    }

    fn read_arboard_image() -> Result<ClipboardImagePayload, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|err| err.to_string())?;
        let image = clipboard.get_image().map_err(|err| err.to_string())?;
        let bytes = Self::rgba_to_png_bytes(image.width, image.height, &image.bytes)?;
        Ok(ClipboardImagePayload {
            bytes,
            extension: "png",
        })
    }

    fn read_arboard_file_image() -> Result<ClipboardImagePayload, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|err| err.to_string())?;
        let files = clipboard.get().file_list().map_err(|err| err.to_string())?;
        let image_path = files
            .into_iter()
            .find_map(|path| Self::supported_image_extension(&path).map(|ext| (path, ext)))
            .ok_or_else(|| "clipboard file list contains no supported image".to_string())?;
        let bytes = std::fs::read(&image_path.0).map_err(|err| err.to_string())?;
        Ok(ClipboardImagePayload {
            bytes,
            extension: image_path.1,
        })
    }

    pub(super) fn rgba_to_png_bytes(
        width: usize,
        height: usize,
        rgba: &[u8],
    ) -> Result<Vec<u8>, String> {
        let expected_len = width
            .checked_mul(height)
            .and_then(|px| px.checked_mul(RGBA_CHANNEL_COUNT))
            .ok_or_else(|| "clipboard image dimensions overflowed".to_string())?;
        if rgba.len() != expected_len {
            return Err(format!(
                "clipboard image bytes length mismatch: expected {expected_len}, got {}",
                rgba.len()
            ));
        }

        let mut out_bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut out_bytes);
        encoder
            .write_image(
                rgba,
                width as u32,
                height as u32,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|err| err.to_string())?;
        Ok(out_bytes)
    }

    fn supported_image_extension(path: &Path) -> Option<&'static str> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "png" => Some("png"),
            "jpg" => Some("jpg"),
            "jpeg" => Some("jpeg"),
            "gif" => Some("gif"),
            "webp" => Some("webp"),
            "bmp" => Some("bmp"),
            _ => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rgba_to_png_bytes_encodes_png() {
        let bytes =
            ClipboardImageOps::rgba_to_png_bytes(1, 1, &[255, 0, 0, 255]).expect("png encodes");
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn rgba_to_png_bytes_rejects_bad_length() {
        let err = ClipboardImageOps::rgba_to_png_bytes(2, 2, &[255, 0, 0, 255])
            .expect_err("bad length must fail");
        assert!(err.contains("length mismatch"));
    }

    #[test]
    fn supported_image_extension_accepts_uppercase_images() {
        let path = PathBuf::from("/tmp/photo.PNG");
        assert_eq!(
            ClipboardImageOps::supported_image_extension(&path),
            Some("png")
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_clipboard_info_detects_supported_image_type() {
        assert!(
            ClipboardImageOps::macos_clipboard_info_references_supported_image(
                "«class PNGf», 132194, JPEG picture, 132194"
            )
        );
        assert!(
            !ClipboardImageOps::macos_clipboard_info_references_supported_image(
                "«class HTML», 4895, «class utf8», 202"
            )
        );
    }
}
