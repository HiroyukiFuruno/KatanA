use super::types::DiagramThemeSnapshot;
use katana_diagram_renderer::{RenderThemeMode, RenderThemeSnapshot};

pub(super) struct KdrThemeAdapter;

impl KdrThemeAdapter {
    pub(super) fn convert(theme: &DiagramThemeSnapshot) -> RenderThemeSnapshot {
        RenderThemeSnapshot {
            mode: if theme.is_dark {
                RenderThemeMode::Dark
            } else {
                RenderThemeMode::Light
            },
            background: theme.background.clone(),
            text: theme.text.clone(),
            fill: theme.fill.clone(),
            stroke: theme.stroke.clone(),
            arrow: theme.arrow.clone(),
            drawio_label_color: theme.drawio_label_color.clone(),
            mermaid_theme: theme.mermaid_theme.clone(),
            plantuml_class_bg: theme.plantuml_class_background.clone(),
            plantuml_note_bg: theme.plantuml_note_background.clone(),
            plantuml_note_text: theme.plantuml_note_text.clone(),
            syntax_theme_dark: theme.syntax_theme_dark.clone(),
            syntax_theme_light: theme.syntax_theme_light.clone(),
            preview_text: theme.preview_text.clone(),
        }
    }
}
