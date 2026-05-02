const TEXT_FONT_FAMILY: &str = "Hiragino Sans, Hiragino Kaku Gothic ProN, Yu Gothic, Meiryo, Noto Sans CJK JP, Noto Sans CJK, DejaVu Sans, Arial, sans-serif";

pub(crate) struct NativeTextRuns;

impl NativeTextRuns {
    pub(crate) fn font_family() -> &'static str {
        TEXT_FONT_FAMILY
    }

    pub(crate) fn render(line: &str) -> String {
        let text = line.chars().filter(|it| !is_emoji(*it)).collect::<String>();
        escape_xml(&text)
    }
}

fn is_emoji(character: char) -> bool {
    matches!(
        character as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF
    )
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::NativeTextRuns;

    #[test]
    fn render_strips_decorative_emoji_without_dropping_text() {
        assert_eq!(
            NativeTextRuns::render("✅ Verification Complete"),
            " Verification Complete"
        );
    }
}
