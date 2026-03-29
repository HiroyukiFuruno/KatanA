mod constants;
mod legacy_types;

use crate::theme::migration::constants::*;
use crate::theme::migration::legacy_types::{ThemeColorsDef, ThemeColorsLegacyData};
use crate::theme::types::{CodeColors, PreviewColors, SystemColors, ThemeColors, ThemeMode};
use serde::{Deserialize, Deserializer};

// ── Deserialize implementation ──────────────────────────────────

impl<'de> Deserialize<'de> for ThemeColors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_theme_colors(deserializer)
    }
}

pub(crate) fn deserialize_theme_colors<'de, D>(deserializer: D) -> Result<ThemeColors, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Migrator {
        New(#[serde(with = "ThemeColorsDef")] ThemeColors),
        Legacy(ThemeColorsLegacyData),
    }

    match Migrator::deserialize(deserializer)? {
        Migrator::New(c) => Ok(c),
        Migrator::Legacy(l) => {
            let is_dark = l.mode == ThemeMode::Dark;
            Ok(ThemeColors {
                name: l.name,
                mode: l.mode,
                system: SystemColors {
                    background: l.background,
                    panel_background: l.panel_background,
                    text: l.text,
                    text_secondary: l.text_secondary,
                    success_text: if is_dark {
                        LEGACY_SUCCESS_DARK
                    } else {
                        LEGACY_SUCCESS_LIGHT
                    },
                    warning_text: l.warning_text,
                    error_text: l.error_text,
                    accent: l.accent,
                    title_bar_text: l.title_bar_text,
                    file_tree_text: l.file_tree_text,
                    active_file_highlight: l.active_file_highlight,
                    button_background: if is_dark {
                        LEGACY_BUTTON_BG_DARK
                    } else {
                        LEGACY_BUTTON_BG_LIGHT
                    },
                    button_active_background: if is_dark {
                        LEGACY_BUTTON_ACTIVE_DARK
                    } else {
                        LEGACY_BUTTON_ACTIVE_LIGHT
                    },
                    border: l.border,
                    selection: l.selection,
                },
                code: CodeColors {
                    background: l.code_background,
                    text: l.text,
                    line_number_text: if is_dark {
                        LEGACY_LINE_NUMBER_DARK
                    } else {
                        LEGACY_LINE_NUMBER_LIGHT
                    },
                    line_number_active_text: l.text,
                    current_line_background: if is_dark {
                        LEGACY_CURRENT_LINE_DARK
                    } else {
                        LEGACY_CURRENT_LINE_LIGHT
                    },
                    hover_line_background: if is_dark {
                        LEGACY_HOVER_LINE_DARK
                    } else {
                        LEGACY_HOVER_LINE_LIGHT
                    },
                    selection: l.selection,
                },
                preview: PreviewColors {
                    background: l.preview_background,
                    text: l.text,
                    warning_text: l.warning_text,
                    border: l.border,
                    selection: l.selection,
                    hover_line_background: if is_dark {
                        LEGACY_HOVER_LINE_DARK
                    } else {
                        LEGACY_HOVER_LINE_LIGHT
                    },
                },
            })
        }
    }
}
