//! A commonmark viewer for egui
//!
//! # Example
//!
//! ```
//! # use egui_commonmark::*;
//! # use egui::__run_test_ui;
//! let markdown =
//! r"# Hello world
//!
//! * A list
//! * [ ] Checkbox
//! ";
//!
//! # __run_test_ui(|ui| {
//! let mut cache = CommonMarkCache::default();
//! CommonMarkViewer::new().show(ui, &mut cache, markdown);
//! # });
//!
//! ```
//!
//! Remember to opt into the image formats you want to use!
//!
//! ```toml
//! image = { version = "0.25", default-features = false, features = ["png"] }
//! ```
//! # FAQ
//!
//! ## URL is not displayed when hovering over a link
//!
//! By default egui does not show urls when you hover hyperlinks. To enable it,
//! you can do the following before calling any ui related functions:
//!
//! ```
//! # use egui::__run_test_ui;
//! # __run_test_ui(|ui| {
//! ui.style_mut().url_in_tooltip = true;
//! # });
//! ```
//!
//!
//! # Compile time evaluation of markdown
//!
//! If you want to embed markdown directly the binary then you can enable the `macros` feature.
//! This will do the parsing of the markdown at compile time and output egui widgets.
//!
//! ## Example
//!
//! ```
//! use egui_commonmark::{CommonMarkCache, commonmark};
//! # egui::__run_test_ui(|ui| {
//! let mut cache = CommonMarkCache::default();
//! let _response = commonmark!(ui, &mut cache, "# ATX Heading Level 1");
//! # });
//! ```
//!
//! Alternatively you can embed a file
//!
//!
//! ## Example
//!
//! ```rust,ignore
//! use egui_commonmark::{CommonMarkCache, commonmark_str};
//! # egui::__run_test_ui(|ui| {
//! let mut cache = CommonMarkCache::default();
//! commonmark_str!(ui, &mut cache, "content.md");
//! # });
//! ```
//!
//! For more information check out the documentation for
//! [egui_commonmark_macros](https://docs.rs/crate/egui_commonmark_macros/latest)
#![cfg_attr(feature = "document-features", doc = "# Features")]
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]

use egui::{self, Id};

pub(crate) mod parsers;
pub mod ui_components;

pub use egui_commonmark_backend::RenderHtmlFn;
pub use egui_commonmark_backend::RenderMathFn;
pub use egui_commonmark_backend::RenderTableFn;
pub use egui_commonmark_backend::RenderTextFn;
pub use egui_commonmark_backend::alerts::{Alert, AlertBundle};
pub use egui_commonmark_backend::misc::{CommonMarkCache, CommonMarkOptions};
pub use egui_commonmark_backend::{
    bullet_point, bullet_point_hollow, newline, number_point,
};
pub use egui_commonmark_backend::pulldown::{EventIteratorItem, Table};

/// An action emitted when a user interacts with a task list checkbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListAction {
    pub span: std::ops::Range<usize>,
    pub new_state: char,
}

#[cfg(feature = "better_syntax_highlighting")]
pub use egui_commonmark_backend::syntect;

#[cfg(feature = "macros")]
pub use egui_commonmark_macros::*;

#[cfg(feature = "macros")]
// Do not rely on this directly!
#[doc(hidden)]
pub use egui_commonmark_backend;


pub struct CommonMarkViewer<'f> {
    options: CommonMarkOptions<'f>,
    heading_anchors: Option<&'f mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    /// Captures the rendered bounding boxes of specific blocks (Diagrams, Alerts, Tables)
    /// associated with their original source spans. Used for stable split-sync anchors.
    block_anchors: Option<&'f mut Vec<(std::ops::Range<usize>, egui::Rect)>>,
    heading_offset: usize,
    active_char_range: Option<std::ops::Range<usize>>,
    hovered_spans: Option<&'f mut Vec<std::ops::Range<usize>>>,
    active_bg_color: Option<egui::Color32>,
    hover_bg_color: Option<egui::Color32>,
    custom_task_box_fn: Option<
        &'f dyn Fn(&mut egui::Ui, char, std::ops::Range<usize>, bool, &mut Vec<TaskListAction>),
    >,
    custom_emoji_fn: Option<&'f dyn Fn(&str, u32) -> Option<Vec<u8>>>,
    custom_task_context_menu_fn: Option<
        &'f dyn Fn(&egui::Response, char, std::ops::Range<usize>, bool, &mut Vec<TaskListAction>),
    >,
    /// Called after a list item is fully rendered. Receives the correct bounding rect
    /// (from `horizontal_wrapped` response) and the item's source span.
    /// Returns `(active_highlighted, hovered)` so the renderer can update internal bookkeeping.
    custom_list_item_highlight_fn:
        Option<&'f dyn Fn(&mut egui::Ui, egui::Rect, &std::ops::Range<usize>) -> (bool, bool)>,
    search_query: Option<String>,
    search_scroll_pending: bool,
    search_active_match_index: Option<usize>,
    search_match_offset: Option<&'f mut usize>,
}

impl<'f> Default for CommonMarkViewer<'f> {
    fn default() -> Self {
        Self {
            options: Default::default(),
            heading_anchors: None,
            block_anchors: None,
            heading_offset: 0,
            active_char_range: None,
            hovered_spans: None,
            active_bg_color: None,
            hover_bg_color: None,
            custom_task_box_fn: None,
            custom_emoji_fn: None,
            custom_task_context_menu_fn: None,
            custom_list_item_highlight_fn: None,
            search_query: None,
            search_scroll_pending: false,
            search_active_match_index: None,
            search_match_offset: None,
        }
    }
}

impl<'f> CommonMarkViewer<'f> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn custom_text_fn(
        mut self,
        func: Option<&'f RenderTextFn>,
    ) -> Self {
        self.options.custom_text_fn = func;
        self
    }

    pub fn custom_task_box_fn(
        mut self,
        func: Option<
            &'f dyn Fn(
                &mut egui::Ui,
                char,
                std::ops::Range<usize>,
                bool,
                &mut std::vec::Vec<TaskListAction>,
            ),
        >,
    ) -> Self {
        self.custom_task_box_fn = func;
        self
    }

    pub fn custom_emoji_fn(
        mut self,
        func: Option<&'f dyn Fn(&str, u32) -> Option<std::vec::Vec<u8>>>,
    ) -> Self {
        self.custom_emoji_fn = func;
        self
    }

    pub fn custom_task_context_menu_fn(
        mut self,
        func: Option<
            &'f dyn Fn(
                &egui::Response,
                char,
                std::ops::Range<usize>,
                bool,
                &mut std::vec::Vec<TaskListAction>,
            ),
        >,
    ) -> Self {
        self.custom_task_context_menu_fn = func;
        self
    }

    /// Set a callback for list item highlight/hover rendering.
    /// The callback receives the correct bounding rect and the item's source span.
    /// It should paint any highlight/hover visuals and return `(active_highlighted, hovered)`.
    pub fn custom_list_item_highlight_fn(
        mut self,
        func: Option<
            &'f dyn Fn(&mut egui::Ui, egui::Rect, &std::ops::Range<usize>) -> (bool, bool),
        >,
    ) -> Self {
        self.custom_list_item_highlight_fn = func;
        self
    }

    pub fn heading_anchors(
        mut self,
        anchors: &'f mut Vec<(std::ops::Range<usize>, egui::Rect)>,
    ) -> Self {
        self.heading_anchors = Some(anchors);
        self
    }

    pub fn block_anchors(
        mut self,
        anchors: &'f mut Vec<(std::ops::Range<usize>, egui::Rect)>,
    ) -> Self {
        self.block_anchors = Some(anchors);
        self
    }

    pub fn active_char_range(mut self, range: std::ops::Range<usize>) -> Self {
        self.active_char_range = Some(range);
        self
    }

    pub fn hovered_spans(mut self, spans: &'f mut Vec<std::ops::Range<usize>>) -> Self {
        self.hovered_spans = Some(spans);
        self
    }

    pub fn active_bg_color(mut self, color: Option<egui::Color32>) -> Self {
        self.active_bg_color = color;
        self
    }

    pub fn hover_bg_color(mut self, color: Option<egui::Color32>) -> Self {
        self.hover_bg_color = color;
        self
    }

    pub fn heading_offset(mut self, offset: usize) -> Self {
        self.heading_offset = offset;
        self
    }

    pub fn search_query(mut self, query: Option<String>) -> Self {
        self.search_query = query;
        self
    }

    pub fn search_active_match_index(mut self, index: usize) -> Self {
        self.search_active_match_index = Some(index);
        self
    }

    pub fn search_match_offset(mut self, offset: &'f mut usize) -> Self {
        self.search_match_offset = Some(offset);
        self
    }

    pub fn search_scroll_pending(mut self, pending: bool) -> Self {
        self.search_scroll_pending = pending;
        self
    }

    /// The amount of spaces a bullet point is indented. By default this is 4
    /// spaces.
    pub fn indentation_spaces(mut self, spaces: usize) -> Self {
        self.options.indentation_spaces = spaces;
        self
    }

    /// The maximum size images are allowed to be. They will be scaled down if
    /// they are larger
    pub fn max_image_width(mut self, width: Option<usize>) -> Self {
        self.options.max_image_width = width;
        self
    }

    /// The default width of the ui. This is only respected if this is larger than
    /// the [`max_image_width`](Self::max_image_width)
    pub fn default_width(mut self, width: Option<usize>) -> Self {
        self.options.default_width = width;
        self
    }

    /// Show alt text when hovering over images. By default this is enabled.
    pub fn show_alt_text_on_hover(mut self, show: bool) -> Self {
        self.options.show_alt_text_on_hover = show;
        self
    }

    /// Allows changing the default implicit `file://` uri scheme.
    /// This does nothing if [`explicit_image_uri_scheme`](`Self::explicit_image_uri_scheme`) is enabled
    ///
    /// # Example
    /// ```
    /// # use egui_commonmark::CommonMarkViewer;
    /// CommonMarkViewer::new().default_implicit_uri_scheme("https://example.org/");
    /// ```
    pub fn default_implicit_uri_scheme<S: Into<String>>(mut self, scheme: S) -> Self {
        self.options.default_implicit_uri_scheme = scheme.into();
        self
    }

    /// Whether to show the code block copy button. Default is true.
    pub fn show_code_copy_button(mut self, show: bool) -> Self {
        self.options.show_code_copy_button = show;
        self
    }

    /// By default any image without a uri scheme such as `foo://` is assumed to
    /// be of the type `file://`. This assumption can sometimes be wrong or be done
    /// incorrectly, so if you want to always be explicit with the scheme then set
    /// this to `true`
    pub fn explicit_image_uri_scheme(mut self, use_explicit: bool) -> Self {
        self.options.use_explicit_uri_scheme = use_explicit;
        self
    }

    #[cfg(feature = "better_syntax_highlighting")]
    /// Set the syntax theme to be used inside code blocks in light mode
    pub fn syntax_theme_light<S: Into<String>>(mut self, theme: S) -> Self {
        self.options.theme_light = theme.into();
        self
    }

    #[cfg(feature = "better_syntax_highlighting")]
    /// Set the syntax theme to be used inside code blocks in dark mode
    pub fn syntax_theme_dark<S: Into<String>>(mut self, theme: S) -> Self {
        self.options.theme_dark = theme.into();
        self
    }

    /// Specify what kind of alerts are supported. This can also be used to localize alerts.
    ///
    /// By default [github flavoured markdown style alerts](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts)
    /// are used
    pub fn alerts(mut self, alerts: AlertBundle) -> Self {
        self.options.alerts = alerts;
        self
    }

    /// Allows rendering math. This has to be done manually as you might want a different
    /// implementation for the web and native.
    ///
    /// The example is template code for rendering a svg image. Make sure to enable the
    /// `egui_extras/svg` feature for the result to show up.
    ///
    /// ## Example
    ///
    /// ```
    /// # use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
    /// # use egui_commonmark::CommonMarkViewer;
    /// let mut math_images = Rc::new(RefCell::new(HashMap::new()));
    /// CommonMarkViewer::new()
    ///     .render_math_fn(Some(&move |ui, math, inline| {
    ///         let mut map = math_images.borrow_mut();
    ///         let svg = map
    ///             .entry(math.to_string())
    ///             .or_insert_with(|| {
    ///                 if inline {
    ///                     // render as inline
    ///                     // dummy data for the example
    ///                     Arc::new([0])
    ///                 } else {
    ///                     Arc::new([0])
    ///                 }
    ///             });
    ///
    ///     let uri = format!("{}.svg", egui::Id::from(math.to_string()).value());
    ///     ui.add(
    ///          egui::Image::new(egui::ImageSource::Bytes {
    ///             uri: uri.into(),
    ///             bytes: egui::load::Bytes::Shared(svg.clone()),
    ///          })
    ///          .fit_to_original_size(1.0),
    ///     );
    ///     }));
    /// ```
    pub fn render_math_fn(mut self, func: Option<&'f RenderMathFn>) -> Self {
        self.options.math_fn = func;
        self
    }

    /// Allows custom handling of html. Enabling this will disable plain text rendering
    /// of html blocks. Nodes are included in the provided text
    pub fn render_html_fn(mut self, func: Option<&'f RenderHtmlFn>) -> Self {
        self.options.html_fn = func;
        self
    }

    /// Allows custom handling of tables.
    pub fn render_table_fn(mut self, func: Option<&'f RenderTableFn>) -> Self {
        self.options.table_fn = func;
        self
    }

    /// Whether to render the collected footnotes at the end of the markdown string. Default is true.
    pub fn render_footnotes(mut self, render: bool) -> Self {
        self.options.render_footnotes = render;
        self
    }

    /// Shows rendered markdown
    pub fn show(
        self,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        text: &str,
    ) -> egui::InnerResponse<()> {
        egui_commonmark_backend::prepare_show(cache, ui.ctx());

        let mut internal = parsers::pulldown::CommonMarkViewerInternal::new(
            self.heading_anchors,
            self.block_anchors,
            self.heading_offset,
            self.active_char_range.clone(),
            self.hovered_spans,
            self.active_bg_color,
            self.hover_bg_color,
            self.custom_task_box_fn,
            self.custom_emoji_fn,
            self.custom_task_context_menu_fn,
            self.custom_list_item_highlight_fn,
            self.search_query.clone(),
        );
        internal.search_scroll_pending = self.search_scroll_pending;
        internal.search_active_match_index = self.search_active_match_index;
        if let Some(ref offset) = self.search_match_offset {
            internal.search_match_counter = **offset;
        }

        let (response, _) = internal.show(ui, cache, &self.options, text, None);

        if let Some(offset) = self.search_match_offset {
            *offset = internal.search_match_counter;
        }

        response
    }

    /// Shows rendered markdown, and allows the rendered ui to mutate the source text.
    ///
    /// Checkmarks can be toggled through the ui, supporting standard `[x]` and custom Katana states `[/]`.
    pub fn show_mut(
        mut self,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        text: &mut String,
    ) -> egui::InnerResponse<()> {
        egui_commonmark_backend::prepare_show(cache, ui.ctx());
        self.options.mutable = true;

        let (mut inner_response, checkmark_events) = {
            let mut internal = parsers::pulldown::CommonMarkViewerInternal::new(
                self.heading_anchors,
                self.block_anchors,
                self.heading_offset,
                self.active_char_range.clone(),
                self.hovered_spans,
                self.active_bg_color,
                self.hover_bg_color,
                self.custom_task_box_fn,
                self.custom_emoji_fn,
                self.custom_task_context_menu_fn,
                self.custom_list_item_highlight_fn,
                self.search_query.clone(),
            );
            internal.search_scroll_pending = self.search_scroll_pending;
            internal.search_active_match_index = self.search_active_match_index;
            if let Some(ref offset) = self.search_match_offset {
                internal.search_match_counter = **offset;
            }

            let result = internal.show(ui, cache, &self.options, text, None);

            if let Some(offset) = self.search_match_offset {
                *offset = internal.search_match_counter;
            }
            result
        };

        // Update source text for checkmarks that were clicked
        for ev in checkmark_events {
            text.replace_range(ev.span, &format!("[{}]", ev.new_state));
            inner_response.response.mark_changed();
        }

        inner_response
    }
}

/// Helper to accurately map task list events back to the original text.
/// Extracts the span of every task list item `[ ]`, `[x]`, `[/]` in the order they appear.
pub fn extract_task_list_spans(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
    options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = pulldown_cmark::Parser::new_ext(text, options).into_offset_iter();
    // Use the raw events vector to run our pre-pass
    let events: Vec<_> = parser.collect();
    let (processed, _) = crate::parsers::pulldown::extract_custom_task_lists(events);

    let mut spans = Vec::new();
    for (event, span) in processed {
        if let pulldown_cmark::Event::Html(html) = event {
            if html.starts_with("<!-- KATANA_TASK:") {
                spans.push(span);
            }
        }
    }

    // pulldown-cmark sometimes emits the same HTML block multiple times if it wraps?
    // No, our pass shouldn't duplicate. We'll sort and deduplicate just in case.
    spans.sort_by_key(|s| s.start);
    spans.dedup();
    spans
}

impl<'f> CommonMarkViewer<'f> {
    /// Shows parsed markdown and returns any task list action events.
    /// Used when the application needs to handle the text mutation externally.
    pub fn show_with_events(
        mut self,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        text: &str,
    ) -> (egui::InnerResponse<()>, Vec<TaskListAction>) {
        self.options.mutable = true;
        egui_commonmark_backend::prepare_show(cache, ui.ctx());

        let mut internal = parsers::pulldown::CommonMarkViewerInternal::new(
            self.heading_anchors,
            self.block_anchors,
            self.heading_offset,
            self.active_char_range.clone(),
            self.hovered_spans,
            self.active_bg_color,
            self.hover_bg_color,
            self.custom_task_box_fn,
            self.custom_emoji_fn,
            self.custom_task_context_menu_fn,
            self.custom_list_item_highlight_fn,
            self.search_query.clone(),
        );
        internal.search_scroll_pending = self.search_scroll_pending;
        internal.search_active_match_index = self.search_active_match_index;
        if let Some(ref offset) = self.search_match_offset {
            internal.search_match_counter = **offset;
        }

        let result = internal.show(ui, cache, &self.options, text, None);

        if let Some(offset) = self.search_match_offset {
            *offset = internal.search_match_counter;
        }

        result
    }

    /// Shows markdown inside a [`ScrollArea`].
    /// This function is much more performant than just calling [`show`] inside a [`ScrollArea`],
    /// because it only renders elements that are visible.
    ///
    /// # Caveat
    ///
    /// This assumes that the markdown is static. If it does change, you have to clear the cache
    /// by using [`clear_scrollable_with_id`](CommonMarkCache::clear_scrollable_with_id) or
    /// [`clear_scrollable`](CommonMarkCache::clear_scrollable). If the content changes every frame,
    /// it's faster to call [`show`] directly.
    ///
    /// [`ScrollArea`]: egui::ScrollArea
    /// [`show`]: crate::CommonMarkViewer::show
    #[doc(hidden)] // Buggy in scenarios more complex than the example application
    #[cfg(feature = "pulldown_cmark")]
    pub fn show_scrollable(
        self,
        source_id: impl std::hash::Hash,
        ui: &mut egui::Ui,
        cache: &mut CommonMarkCache,
        text: &str,
    ) {
        egui_commonmark_backend::prepare_show(cache, ui.ctx());
        let mut internal = parsers::pulldown::CommonMarkViewerInternal::new(
            self.heading_anchors,
            self.block_anchors,
            self.heading_offset,
            self.active_char_range.clone(),
            self.hovered_spans,
            self.active_bg_color,
            self.hover_bg_color,
            self.custom_task_box_fn,
            self.custom_emoji_fn,
            self.custom_task_context_menu_fn,
            self.custom_list_item_highlight_fn,
            self.search_query.clone(),
        );
        internal.search_scroll_pending = self.search_scroll_pending;
        internal.search_active_match_index = self.search_active_match_index;
        internal.show_scrollable(Id::new(source_id), ui, cache, &self.options, text);
    }

}

pub(crate) struct ListLevel {
    current_number: Option<u64>,
}

#[derive(Default)]
pub(crate) struct List {
    items: Vec<ListLevel>,
    has_list_begun: bool,
}

impl List {
    pub fn start_level_with_number(&mut self, start_number: u64) {
        self.items.push(ListLevel {
            current_number: Some(start_number),
        });
    }

    pub fn start_level_without_number(&mut self) {
        self.items.push(ListLevel {
            current_number: None,
        });
    }

    pub fn is_inside_a_list(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn is_last_level(&self) -> bool {
        self.items.len() == 1
    }

    pub fn start_item_newline(&mut self, ui: &mut egui::Ui, inside_blockquote: bool) {
        // To ensure that newlines are only inserted within the list and not before it
        if self.has_list_begun {
            if !inside_blockquote {
                newline(ui);
            }
        } else {
            self.has_list_begun = true;
        }
    }

    pub fn start_item_content(
        &mut self,
        ui: &mut egui::Ui,
        options: &CommonMarkOptions,
        is_task_list: bool,
    ) {
        let len = self.items.len();
        if let Some(item) = self.items.last_mut() {
            ui.label(" ".repeat((len - 1) * options.indentation_spaces));

            if let Some(number) = &mut item.current_number {
                number_point(ui, &number.to_string());
                *number += 1;
            } else if !is_task_list && len > 1 {
                bullet_point_hollow(ui);
            } else if !is_task_list {
                bullet_point(ui);
            }
        } else {
            unreachable!();
        }

        ui.add_space(4.0);
    }

    pub fn end_level(&mut self, ui: &mut egui::Ui, insert_newline: bool) {
        self.items.pop();

        if self.items.is_empty() && insert_newline {
            newline(ui);
        }
    }
}
