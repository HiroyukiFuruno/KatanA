use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::{DiagramThemeOverride, DiagramThemeSnapshot};

pub(super) struct ThemeStateRestore {
    previous_dark_mode: bool,
    previous_snapshot: DiagramThemeSnapshot,
}

impl ThemeStateRestore {
    pub(super) fn capture() -> Self {
        Self {
            previous_dark_mode: DiagramColorPreset::is_dark_mode(),
            previous_snapshot: DiagramThemeSnapshot::current(),
        }
    }
}

impl Drop for ThemeStateRestore {
    fn drop(&mut self) {
        DiagramColorPreset::set_dark_mode(self.previous_dark_mode);
        let preset = DiagramColorPreset::current();
        let name = if self.previous_dark_mode {
            "dark"
        } else {
            "light"
        };
        let baseline = DiagramThemeSnapshot::from_preset(name, self.previous_dark_mode, preset);
        if self.previous_snapshot == baseline {
            DiagramThemeSnapshot::clear_current_override();
            return;
        }
        /* WHY: overrideのgetterがなく、有効snapshotしか復元できない。元のoverride有無は復元できない。 */
        DiagramThemeSnapshot::set_current_override(override_from_snapshot(&self.previous_snapshot));
    }
}

fn override_from_snapshot(snapshot: &DiagramThemeSnapshot) -> DiagramThemeOverride {
    DiagramThemeOverride {
        name: snapshot.name.clone(),
        is_dark: snapshot.is_dark,
        background: snapshot.background.clone(),
        text: snapshot.text.clone(),
        preview_text: snapshot.preview_text.clone(),
        table_border: snapshot.table_border.clone(),
        table_header_background: snapshot.table_header_background.clone(),
        table_even_row_background: snapshot.table_even_row_background.clone(),
    }
}
