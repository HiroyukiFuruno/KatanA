use crate::preview_pane::PreviewPane;

pub(crate) const SETTINGS_WINDOW_DEFAULT_WIDTH: f32 = 1000.0;
pub(crate) const SETTINGS_WINDOW_DEFAULT_HEIGHT: f32 = 750.0;
pub(crate) const SETTINGS_SIDE_PANEL_DEFAULT_WIDTH: f32 = 200.0;
pub(crate) const SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH: f32 = 350.0;
pub(crate) const SETTINGS_HEADER_FONT_SIZE: f32 = 14.0;
pub(crate) const SETTINGS_GROUP_SPACING: f32 = 8.0;
pub(crate) const SETTINGS_TOGGLE_SPACING: f32 = 8.0;
pub(crate) const AUTO_SAVE_INTERVAL_MIN: f64 = 0.0;
pub(crate) const AUTO_SAVE_INTERVAL_MAX: f64 = 300.0;
pub(crate) const AUTO_SAVE_INTERVAL_STEP: f64 = 0.1;
pub(crate) const SECTION_SPACING: f32 = 12.0;
pub(crate) const SUBSECTION_SPACING: f32 = 6.0;
pub(crate) const INNER_MARGIN: f32 = 12.0;
pub(crate) const FONT_SIZE_STEP: f64 = 1.0;
pub(crate) const LAYOUT_SELECTOR_SPACING: f32 = 4.0;
pub(crate) const PRESET_SWATCH_SIZE: f32 = 14.0;
pub(crate) const COLOR_GRID_LABEL_WIDTH: f32 = 130.0;
pub(crate) const SECTION_HEADER_SIZE: f32 = 14.0;
pub(crate) const SECTION_HEADER_MARGIN: f32 = 4.0;
pub(crate) const SWATCH_CORNER_DIVISOR: f32 = 4.0;
pub(crate) const FONT_FAMILY_COMBOBOX_WIDTH: f32 = 200.0;
pub(crate) const FONT_DROPDOWN_MAX_HEIGHT: f32 = 200.0;
pub(crate) const SLIDER_RAIL_OPACITY: u8 = 80;
pub(crate) const SLIDER_BORDER_WIDTH: f32 = 1.0;
pub(crate) const HINT_FONT_SIZE: f32 = 10.0;

pub(crate) const SAMPLE_MARKDOWN: &str = concat!(
    r#"# Heading 1

## Heading 2

Normal paragraph text with **bold**, *italic*, and `inline code`.

- List item 1
- List item 2
  - Nested item

> Blockquote text goes here.

```rust
fn main() {
    println!("Hello, KatanA!");
}
```

| Column A | Column B |
|----------|----------|
| Cell 1   | Cell 2   |

---

"#,
    "Secondary text and [a link](https://example.com) for reference.\n"
);

pub(crate) struct SettingsWindow<'a> {
    pub state: &'a mut crate::app_state::AppState,
    pub preview_pane: &'a mut PreviewPane,
}

pub struct SettingsOps;
