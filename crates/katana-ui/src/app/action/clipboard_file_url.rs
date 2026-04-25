use super::clipboard_image::ClipboardImagePayload;
use std::path::{Path, PathBuf};

const FILE_URL_PREFIX: &str = "file://";
const FILE_URL_LOCALHOST_PREFIX: &str = "localhost";
const PERCENT_BYTE_PREFIX: u8 = b'%';
const PERCENT_ENCODED_HEX_DIGITS: usize = 2;
const PERCENT_ENCODED_TOTAL_BYTES: usize = 3;
const HEX_HIGH_NIBBLE_SHIFT: u8 = 4;
const DECIMAL_DIGIT_BASE: u8 = 10;

pub(crate) struct ClipboardFileUrlOps;

impl ClipboardFileUrlOps {
    pub(crate) fn read_image_payload() -> Result<ClipboardImagePayload, String> {
        let mut clipboard = arboard::Clipboard::new().map_err(|err| err.to_string())?;
        let text = clipboard.get_text().map_err(|err| err.to_string())?;
        let image_path = Self::first_image_file_url_path(&text)
            .ok_or_else(|| "clipboard text contains no supported file URL image".to_string())?;
        let extension = Self::supported_image_extension(&image_path)
            .ok_or_else(|| "clipboard file URL is not a supported image".to_string())?;
        let bytes = std::fs::read(&image_path).map_err(|err| err.to_string())?;
        Ok(ClipboardImagePayload { bytes, extension })
    }

    fn first_image_file_url_path(text: &str) -> Option<PathBuf> {
        text.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(Self::file_url_to_path)
            .find(|path| Self::supported_image_extension(path).is_some())
    }

    fn file_url_to_path(raw: &str) -> Option<PathBuf> {
        let file_url_path = raw.strip_prefix(FILE_URL_PREFIX)?;
        let decoded = Self::decode_file_url_path(file_url_path);
        Some(PathBuf::from(decoded))
    }

    fn decode_file_url_path(path: &str) -> String {
        let path = path.strip_prefix(FILE_URL_LOCALHOST_PREFIX).unwrap_or(path);
        let mut out = Vec::with_capacity(path.len());
        let bytes = path.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == PERCENT_BYTE_PREFIX
                && i + PERCENT_ENCODED_HEX_DIGITS < bytes.len()
                && let (Some(hi), Some(lo)) = (Self::hex(bytes[i + 1]), Self::hex(bytes[i + 2]))
            {
                out.push((hi << HEX_HIGH_NIBBLE_SHIFT) | lo);
                i += PERCENT_ENCODED_TOTAL_BYTES;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + DECIMAL_DIGIT_BASE),
            b'A'..=b'F' => Some(byte - b'A' + DECIMAL_DIGIT_BASE),
            _ => None,
        }
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

    #[test]
    fn first_image_file_url_path_decodes_image_url() {
        let path =
            ClipboardFileUrlOps::first_image_file_url_path("file:///tmp/my%20image.PNG").unwrap();
        assert_eq!(path, PathBuf::from("/tmp/my image.PNG"));
    }

    #[test]
    fn first_image_file_url_path_ignores_non_images() {
        assert!(ClipboardFileUrlOps::first_image_file_url_path("file:///tmp/readme.md").is_none());
    }
}
