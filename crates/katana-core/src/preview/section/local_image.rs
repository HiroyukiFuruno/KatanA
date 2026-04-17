use crate::preview::types::{ImageSectionOps, PreviewSection};

impl ImageSectionOps {
    pub fn extract_standalone_images(secs: Vec<PreviewSection>) -> Vec<PreviewSection> {
        let mut result = Vec::with_capacity(secs.len());
        for sec in secs {
            if let PreviewSection::Markdown(ref md, _) = sec {
                /* WHY: Split into paragraphs so standalone images can be extracted
                even when embedded within larger markdown sections. */
                Self::split_paragraphs_extracting_images(md, &mut result);
            } else {
                result.push(sec);
            }
        }
        result
    }

    fn count_lines(text: &str) -> usize {
        text.chars().filter(|c| *c == '\n').count()
            + if !text.is_empty() && !text.ends_with('\n') {
                1
            } else {
                0
            }
    }

    fn split_paragraphs_extracting_images(md: &str, out: &mut Vec<PreviewSection>) {
        /* WHY: Split on double-newlines (paragraph boundary) to find standalone image paragraphs. */
        let paragraphs: Vec<&str> = md.split("\n\n").collect();

        if paragraphs.len() <= 1 {
            if let Some((path, alt)) = Self::try_parse_standalone_image(md) {
                let lines = Self::count_lines(md);
                out.push(PreviewSection::LocalImage { path, alt, lines });
            } else if !md.is_empty() {
                let lines = Self::count_lines(md);
                out.push(PreviewSection::Markdown(md.to_string(), lines));
            }
            return;
        }

        let mut md_buf = String::new();
        for (i, para) in paragraphs.iter().enumerate() {
            if i > 0 {
                md_buf.push_str("\n\n");
            }

            if let Some((path, alt)) = Self::try_parse_standalone_image(para) {
                if !md_buf.is_empty() {
                    let lines = Self::count_lines(&md_buf);
                    out.push(PreviewSection::Markdown(std::mem::take(&mut md_buf), lines));
                }
                /* WHY: For LocalImage, we must count the newlines consumed by this entire paragraph chunk + its leading newlines */
                let lines = Self::count_lines(para);
                out.push(PreviewSection::LocalImage { path, alt, lines });
            } else {
                md_buf.push_str(para);
            }
        }
        if !md_buf.is_empty() {
            let lines = Self::count_lines(&md_buf);
            out.push(PreviewSection::Markdown(md_buf, lines));
        }
    }

    fn try_parse_standalone_image(md: &str) -> Option<(String, String)> {
        let trimmed = md.trim();
        if trimmed.starts_with("![") && trimmed.ends_with(')') {
            let alt_end = trimmed.find("](")?;
            let alt = trimmed[2..alt_end].to_string();
            let path = trimmed[alt_end + 2..trimmed.len() - 1].to_string();
            Some((path, alt))
        } else {
            None
        }
    }
}
