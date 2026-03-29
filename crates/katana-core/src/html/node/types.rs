//! HTML element node model types.

use std::path::PathBuf;

/// Whether an HTML element creates line breaks (block) or flows inline.
///
/// Corresponds to the CSS `display` property for standard HTML elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Block elements (`<div>`, `<p>`, `<h1>`–`<h6>`) generate line breaks
    /// before and after.
    Block,
    /// Inline elements (`<a>`, `<img>`, `<span>`, `<em>`, `<strong>`)
    /// flow horizontally without line breaks.
    Inline,
}

/// Horizontal text alignment for block elements (e.g. `<p align="center">`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Classification of a link destination.
///
/// Determined at parse time from the `href` attribute or markdown link URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    /// External URL (`http://` or `https://`).
    External(String),
    /// Internal file link (relative path resolved to absolute).
    InternalFile(PathBuf),
    /// Same-document anchor (`#section-id`).
    Anchor(String),
}

/// How a link should be opened when clicked.
///
/// New variants (e.g. `OpenInNewTab`, `OpenInSplitView`) can be added here
/// as navigation features are implemented. The compiler's exhaustive match
/// checking ensures all call sites handle the new variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAction {
    /// Open in the system's default web browser.
    OpenInBrowser(String),
    /// Navigate within the current editor tab (supports history back/forward).
    NavigateCurrentTab(PathBuf),
}

/// A parsed HTML/Markdown element in the preview tree.
///
/// This enum represents the subset of HTML elements that can appear within
/// Markdown content. It is UI-independent and lives in `katana-core`.
#[derive(Debug, Clone, PartialEq)]
pub enum HtmlNode {
    /// Plain text content.
    Text(String),

    /// Image element (`<img>` or `![alt](src)`).
    Image {
        /// Image source URL or file path.
        src: String,
        /// Alt text for accessibility.
        alt: String,
    },

    /// Link element (`<a>` or `[text](url)`).
    Link {
        /// Classified link destination.
        target: LinkTarget,
        /// Child nodes (text, images, etc.) rendered inside the link.
        children: Vec<HtmlNode>,
    },

    /// Heading element (`<h1>`–`<h6>` or `# heading`).
    Heading {
        /// Heading level (1–6).
        level: u8,
        /// Horizontal alignment (from `align` attribute).
        align: Option<TextAlign>,
        /// Child nodes rendered inside the heading.
        children: Vec<HtmlNode>,
    },

    /// Paragraph element (`<p>`) with optional alignment.
    Paragraph {
        /// Horizontal alignment (from `align` attribute).
        align: Option<TextAlign>,
        /// Child nodes rendered inside the paragraph.
        children: Vec<HtmlNode>,
    },

    /// Line break (`<br>`).
    LineBreak,

    /// Emphasis (`<em>` or `*text*`).
    Emphasis(Vec<HtmlNode>),

    /// Strong emphasis (`<strong>` or `**text**`).
    Strong(Vec<HtmlNode>),
}
