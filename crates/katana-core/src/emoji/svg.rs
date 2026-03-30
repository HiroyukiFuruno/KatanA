#[cfg(target_os = "macos")]
use regex::Regex;
#[cfg(target_os = "macos")]
use std::sync::OnceLock;

#[cfg(target_os = "macos")]
use super::data::is_emoji_scalar;

#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_FAMILY: &str = "Apple Color Emoji";

pub fn prefer_apple_color_emoji_in_svg(svg: &str) -> String {
    #[cfg(not(target_os = "macos"))]
    {
        svg.to_owned()
    }

    #[cfg(target_os = "macos")]
    {
        svg_text_regex()
            .replace_all(svg, |caps: &regex::Captures<'_>| {
                let attrs = caps.name("attrs").map_or("", |m| m.as_str());
                let text = caps.name("text").map_or("", |m| m.as_str());
                if !contains_emoji(text) {
                    caps.get(0)
                        .map_or_else(String::new, |full| full.as_str().to_owned())
                } else {
                    format!("<text{}>{}</text>", ensure_emoji_font_family(attrs), text)
                }
            })
            .into_owned()
    }
}

#[cfg(target_os = "macos")]
fn svg_text_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?s)<text(?P<attrs>[^>]*)>(?P<text>.*?)</text>"#)
            .expect("valid svg text regex")
    })
}

#[cfg(target_os = "macos")]
fn font_family_double_quote_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"font-family\s*=\s*"(?P<family>[^"]*)""#)
            .expect("valid double-quoted font-family regex")
    })
}

#[cfg(target_os = "macos")]
fn font_family_single_quote_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"font-family\s*=\s*'(?P<family>[^']*)'"#)
            .expect("valid single-quoted font-family regex")
    })
}

#[cfg(target_os = "macos")]
fn ensure_emoji_font_family(attrs: &str) -> String {
    #[allow(clippy::useless_vec)]
    for (regex, quote) in vec![
        (font_family_double_quote_regex(), '"'),
        (font_family_single_quote_regex(), '\''),
    ] {
        if let Some(caps) = regex.captures(attrs) {
            let family = caps.name("family").map_or("", |m| m.as_str());
            if family.contains(APPLE_COLOR_EMOJI_FONT_FAMILY) {
                return attrs.to_owned();
            }

            let replacement = format!(
                "font-family={quote}{font_family}, {family}{quote}",
                font_family = APPLE_COLOR_EMOJI_FONT_FAMILY,
            );
            return regex.replace(attrs, replacement).into_owned();
        }
    }

    format!(r#"{attrs} font-family="{APPLE_COLOR_EMOJI_FONT_FAMILY}""#)
}

#[cfg(target_os = "macos")]
fn contains_emoji(text: &str) -> bool {
    text.chars().any(is_emoji_scalar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn prefer_apple_color_emoji_in_svg_prefixes_existing_font_family() {
        let svg = r#"<svg><text x="10" font-family="Verdana" font-size="14">❤️</text></svg>"#;

        let processed = prefer_apple_color_emoji_in_svg(svg);

        assert!(processed.contains(r#"font-family="Apple Color Emoji, Verdana""#));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn prefer_apple_color_emoji_in_svg_adds_font_family_when_missing() {
        let svg = r#"<svg><text x="10">❤️ Sponsor</text></svg>"#;

        let processed = prefer_apple_color_emoji_in_svg(svg);

        assert!(processed.contains(r#"font-family="Apple Color Emoji""#));
    }

    #[test]
    fn prefer_apple_color_emoji_in_svg_leaves_plain_text_unchanged() {
        let svg = r#"<svg><text x="10" font-family="Verdana">Sponsor</text></svg>"#;

        let processed = prefer_apple_color_emoji_in_svg(svg);

        assert_eq!(processed, svg);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn prefer_apple_color_emoji_in_svg_preserves_existing_emoji_font_family() {
        let svg = r#"<svg><text x="10" font-family="Apple Color Emoji, Verdana">❤️</text></svg>"#;
        let processed = prefer_apple_color_emoji_in_svg(svg);
        assert!(processed.contains(r#"font-family="Apple Color Emoji, Verdana""#));
    }
}
