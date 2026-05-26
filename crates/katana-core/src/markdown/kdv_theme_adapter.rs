use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::diagram_backend::DiagramThemeSnapshot;
use katana_document_viewer::{KdvThemeMode, KdvThemeSnapshot};
use katana_markdown_model::DiagramKind as KdvDiagramKind;

pub(crate) struct KdvThemeAdapter;

impl KdvThemeAdapter {
    pub(crate) fn from_preset(preset: &DiagramColorPreset) -> KdvThemeSnapshot {
        let name = if preset.dark_mode { "dark" } else { "light" };
        let theme = DiagramThemeSnapshot::from_preset(name, preset.dark_mode, preset);
        Self::from_diagram_theme(&theme)
    }

    pub(crate) fn from_diagram_theme(theme: &DiagramThemeSnapshot) -> KdvThemeSnapshot {
        let mut snapshot = if theme.is_dark {
            KdvThemeSnapshot::katana_dark()
        } else {
            KdvThemeSnapshot::katana_light()
        };
        snapshot.name = theme.name.clone();
        snapshot.mode = if theme.is_dark {
            KdvThemeMode::Dark
        } else {
            KdvThemeMode::Light
        };
        snapshot.background = theme.background.clone();
        snapshot.text = theme.text.clone();
        snapshot.diagram_background = theme.background.clone();
        snapshot.diagram_text = theme.preview_text.clone();
        snapshot.diagram_fill = theme.fill.clone();
        snapshot.diagram_stroke = theme.stroke.clone();
        snapshot.diagram_arrow = theme.arrow.clone();
        snapshot.mermaid_theme = theme.mermaid_theme.clone();
        snapshot.syntax_theme_dark = theme.syntax_theme_dark.clone();
        snapshot.syntax_theme_light = theme.syntax_theme_light.clone();
        snapshot
    }

    pub(crate) fn from_diagram_theme_for_kind(
        theme: &DiagramThemeSnapshot,
        kind: &KdvDiagramKind,
    ) -> KdvThemeSnapshot {
        let mut snapshot = Self::from_diagram_theme(theme);
        if matches!(kind, KdvDiagramKind::PlantUml) {
            snapshot.diagram_fill = theme.plantuml_class_background.clone();
            snapshot.alert_background = theme.plantuml_note_background.clone();
            snapshot.diagram_text = theme.plantuml_note_text.clone();
        }
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_diagram_theme_preserves_light_diagram_snapshot() {
        let preset = DiagramColorPreset::light();
        let source = DiagramThemeSnapshot::from_preset("light", false, preset);
        let theme = KdvThemeAdapter::from_diagram_theme(&source);

        assert_eq!(theme.mode, KdvThemeMode::Light);
        assert_eq!(theme.diagram_background, preset.background);
        assert_eq!(theme.diagram_text, preset.preview_text);
        assert_eq!(theme.diagram_fill, preset.fill);
        assert_eq!(theme.diagram_stroke, preset.stroke);
        assert_eq!(theme.diagram_arrow, preset.arrow);
        assert_eq!(theme.mermaid_theme, preset.mermaid_theme);
    }

    #[test]
    fn from_diagram_theme_for_plantuml_preserves_plantuml_snapshot() {
        let preset = DiagramColorPreset::light();
        let source = DiagramThemeSnapshot::from_preset("light", false, preset);
        let theme =
            KdvThemeAdapter::from_diagram_theme_for_kind(&source, &KdvDiagramKind::PlantUml);

        assert_eq!(theme.diagram_fill, preset.plantuml_class_bg);
        assert_eq!(theme.alert_background, preset.plantuml_note_bg);
        assert_eq!(theme.diagram_text, preset.plantuml_note_text);
    }
}
