use crate::html::node::{HtmlNode, TextAlign};
use ::regex::{Captures, Match, Regex};

use super::{HtmlParser, HtmlRegexOps};

type HtmlTableRow = Vec<Vec<HtmlNode>>;

impl<'a> HtmlParser<'a> {
    pub(super) fn try_parse_paragraph(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::p().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }

        let attrs = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
        let inner = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        let align =
            HtmlRegexOps::extract_attr(attrs, "align").and_then(|a: String| match a.as_str() {
                "center" => Some(TextAlign::Center),
                "left" => Some(TextAlign::Left),
                "right" => Some(TextAlign::Right),
                _ => None,
            });
        let children = self.parse_fragment(inner);
        Some((
            HtmlNode::Paragraph { align, children },
            caps.get(0).unwrap().end(),
        ))
    }

    pub(super) fn try_parse_details(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::details().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }

        let inner = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
        let (summary, body) = self.extract_details_parts(inner)?;
        Some((
            HtmlNode::Details {
                summary,
                children: self.parse_fragment(&body),
            },
            caps.get(0).unwrap().end(),
        ))
    }

    fn extract_details_parts(&self, inner: &str) -> Option<(Vec<HtmlNode>, String)> {
        if let Some(summary_caps) = HtmlRegexOps::summary().captures(inner) {
            let summary_html = summary_caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
            let summary_match = summary_caps.get(0)?;
            let mut body = String::with_capacity(inner.len().saturating_sub(summary_match.len()));
            body.push_str(&inner[..summary_match.start()]);
            body.push_str(&inner[summary_match.end()..]);
            return Some((self.parse_fragment(summary_html), body));
        }
        Some((Vec::new(), inner.to_string()))
    }

    pub(super) fn try_parse_table(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::table().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }

        let inner = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
        let (headers, rows) = self.parse_table_rows(inner);

        Some((
            HtmlNode::Table { headers, rows },
            caps.get(0).unwrap().end(),
        ))
    }

    fn parse_table_rows(&self, table_html: &str) -> (HtmlTableRow, Vec<HtmlTableRow>) {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        for row_caps in HtmlRegexOps::tr().captures_iter(table_html) {
            let row_html = row_caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
            self.push_table_row(row_html, &mut headers, &mut rows);
        }
        (headers, rows)
    }

    fn push_table_row(
        &self,
        row_html: &str,
        headers: &mut HtmlTableRow,
        rows: &mut Vec<HtmlTableRow>,
    ) {
        let header_cells = self.parse_cells(row_html, HtmlRegexOps::th());
        let data_cells = self.parse_cells(row_html, HtmlRegexOps::td());

        if !header_cells.is_empty() && headers.is_empty() {
            *headers = header_cells;
            if !data_cells.is_empty() {
                rows.push(data_cells);
            }
            return;
        }

        let mut row = Vec::new();
        row.extend(header_cells);
        row.extend(data_cells);
        if !row.is_empty() {
            rows.push(row);
        }
    }

    fn parse_cells(&self, row_html: &str, cell_re: &Regex) -> Vec<Vec<HtmlNode>> {
        cell_re
            .captures_iter(row_html)
            .map(|caps| {
                let cell_html = caps.get(1).map(|m: Match| m.as_str()).unwrap_or("");
                self.parse_fragment(cell_html)
            })
            .collect()
    }

    pub(super) fn try_parse_heading(&self, s: &str) -> Option<(HtmlNode, usize)> {
        let caps: Captures = HtmlRegexOps::heading().captures(s)?;
        if caps.get(0).map(|m: Match| m.start()).unwrap_or(1) != 0 {
            return None;
        }

        let level: u8 = caps
            .get(1)
            .map(|m: Match| m.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let attrs = caps.get(2).map(|m: Match| m.as_str()).unwrap_or("");
        const CAP_HEADING_INNER: usize = 3;
        let inner = caps
            .get(CAP_HEADING_INNER)
            .map(|m: Match| m.as_str())
            .unwrap_or("");
        let children = self.parse_fragment(inner);
        Some((
            HtmlNode::Heading {
                level,
                align: Self::heading_align(attrs),
                children,
            },
            caps.get(0).unwrap().end(),
        ))
    }

    fn heading_align(attrs: &str) -> Option<TextAlign> {
        if attrs.contains(r#"align="center""#) {
            Some(TextAlign::Center)
        } else if attrs.contains(r#"align="left""#) {
            Some(TextAlign::Left)
        } else if attrs.contains(r#"align="right""#) {
            Some(TextAlign::Right)
        } else {
            None
        }
    }
}
