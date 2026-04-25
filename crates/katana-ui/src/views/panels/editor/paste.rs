use eframe::egui;

const FILE_URL_PREFIX: &str = "file://";
const FILE_URL_LOCALHOST_PREFIX: &str = "localhost";
const PERCENT_BYTE_PREFIX: u8 = b'%';
const PERCENT_ENCODED_HEX_DIGITS: usize = 2;
const PERCENT_ENCODED_TOTAL_BYTES: usize = 3;
const HEX_HIGH_NIBBLE_SHIFT: u8 = 4;
const DECIMAL_DIGIT_BASE: u8 = 10;

pub(super) struct EditorPasteOps;

impl EditorPasteOps {
    pub(super) fn should_ingest_clipboard_image_paste(
        response_has_focus: bool,
        text_changed: bool,
        events: &[egui::Event],
    ) -> bool {
        if !response_has_focus || text_changed {
            return false;
        }

        let mut paste_signal = false;
        let mut text_paste_seen = false;
        for event in events {
            match event {
                egui::Event::Paste(text) => {
                    if text.is_empty() || Self::paste_text_references_image_file(text) {
                        paste_signal = true;
                    } else {
                        text_paste_seen = true;
                    }
                }
                egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    modifiers,
                    ..
                } if *key == egui::Key::V
                    && *pressed
                    && !*repeat
                    && modifiers.command
                    && !modifiers.shift
                    && !modifiers.alt =>
                {
                    paste_signal = true;
                }
                _ => {}
            }
        }

        paste_signal && !text_paste_seen
    }

    fn paste_text_references_image_file(text: &str) -> bool {
        let mut saw_path = false;
        for raw in text.lines().map(str::trim).filter(|s| !s.is_empty()) {
            let Some(file_url_path) = raw.strip_prefix(FILE_URL_PREFIX) else {
                return false;
            };
            saw_path = true;
            let decoded = Self::decode_file_url_path(file_url_path);
            if !Self::path_has_image_extension(std::path::Path::new(&decoded)) {
                return false;
            }
        }
        saw_path
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

    fn path_has_image_extension(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"
                )
            })
            .unwrap_or(false)
    }
}
