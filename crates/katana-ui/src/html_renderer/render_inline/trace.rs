use std::fmt;

const MAX_SOURCE_CHARS: usize = 128;
const MAX_OUTPUT_BYTES: usize = 192;
const TRUNCATION_MARKER: &str = "…";

pub(super) struct BoundedSource<'a> {
    source: &'a str,
}

fn sanitized_len(character: char) -> usize {
    match character {
        '\n' | '\r' | '\t' => 2,
        character if character.is_control() => character.escape_default().map(char::len_utf8).sum(),
        character => character.len_utf8(),
    }
}

fn write_sanitized<W: fmt::Write>(formatter: &mut W, character: char) -> fmt::Result {
    match character {
        '\n' => formatter.write_str("\\n"),
        '\r' => formatter.write_str("\\r"),
        '\t' => formatter.write_str("\\t"),
        character if character.is_control() => {
            for escaped in character.escape_default() {
                formatter.write_char(escaped)?;
            }
            Ok(())
        }
        character => formatter.write_char(character),
    }
}

impl<'a> BoundedSource<'a> {
    pub(super) const fn new(source: &'a str) -> Self {
        Self { source }
    }
}

impl fmt::Display for BoundedSource<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = 0;
        let mut truncated = false;
        for (source_chars, character) in self.source.chars().enumerate() {
            if source_chars == MAX_SOURCE_CHARS {
                truncated = true;
                break;
            }
            let sanitized_bytes = sanitized_len(character);
            if written + sanitized_bytes + TRUNCATION_MARKER.len() > MAX_OUTPUT_BYTES {
                truncated = true;
                break;
            }
            write_sanitized(formatter, character)?;
            written += sanitized_bytes;
        }
        if truncated {
            formatter.write_str(TRUNCATION_MARKER)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoundedSource, MAX_OUTPUT_BYTES, MAX_SOURCE_CHARS, sanitized_len, write_sanitized,
    };
    use std::fmt::{self, Write};

    struct RejectingWriter;

    impl Write for RejectingWriter {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    struct BudgetWriter {
        remaining: usize,
    }

    impl Write for BudgetWriter {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            if value.len() > self.remaining {
                return Err(fmt::Error);
            }
            self.remaining -= value.len();
            Ok(())
        }
    }

    #[test]
    fn propagates_truncation_marker_write_errors() {
        let source = "x".repeat(MAX_SOURCE_CHARS + 1);
        let mut writer = BudgetWriter {
            remaining: MAX_SOURCE_CHARS,
        };
        assert_eq!(
            write!(writer, "{}", BoundedSource::new(&source)),
            Err(fmt::Error)
        );
    }

    #[test]
    fn fills_utf8_output_limit_without_splitting_a_character() {
        let source = "\u{753b}".repeat(MAX_SOURCE_CHARS);
        let output = BoundedSource::new(&source).to_string();
        assert_eq!(output, format!("{}…", "\u{753b}".repeat(63)));
        assert_eq!(output.len(), MAX_OUTPUT_BYTES);
    }

    #[test]
    fn sanitized_length_matches_actual_control_output() {
        for character in ('\0'..='\u{9f}').chain(['\u{753b}', '\u{1f600}']) {
            let mut output = String::new();
            assert_eq!(write_sanitized(&mut output, character), Ok(()));
            assert_eq!(sanitized_len(character), output.len());
        }
    }

    #[test]
    fn propagates_writer_errors_without_fallback() {
        for source in ["image", "\n", "\r", "\t", "\u{7f}"] {
            let result = write!(RejectingWriter, "{}", BoundedSource::new(source));
            assert_eq!(result, Err(fmt::Error));
        }
    }

    #[test]
    fn preserves_short_normal_urls() {
        let source = "https://example.com/image.svg";
        assert_eq!(BoundedSource::new(source).to_string(), source);
    }

    #[test]
    fn preserves_empty_and_unicode_sources() {
        assert_eq!(BoundedSource::new("").to_string(), "");
        let source = "\u{753b}\u{50cf}\u{1f600}";
        assert_eq!(BoundedSource::new(source).to_string(), source);
    }

    #[test]
    fn bounds_large_data_uri_without_invalid_utf8() {
        let source = format!("data:image/svg+xml,{}", "\u{3042}".repeat(10_000));
        let output = BoundedSource::new(&source).to_string();
        assert!(output.ends_with('…'));
        assert!(output.len() <= MAX_OUTPUT_BYTES);
        assert!(output.chars().count() <= MAX_SOURCE_CHARS + 1);
    }

    #[test]
    fn marks_only_sources_over_the_character_limit() {
        let exact = "x".repeat(MAX_SOURCE_CHARS);
        assert!(!BoundedSource::new(&exact).to_string().ends_with('…'));
        let over = format!("{exact}x");
        assert!(BoundedSource::new(&over).to_string().ends_with('…'));
    }

    #[test]
    fn escapes_controls_to_one_line() {
        let output = BoundedSource::new("image\n\r\t\u{7f}").to_string();
        assert!(!output.contains('\n'));
        assert!(!output.contains('\r'));
        assert!(!output.contains('\t'));
        assert!(output.contains("\\n"));
        assert!(output.contains("\\r"));
        assert!(output.contains("\\t"));
    }
}
