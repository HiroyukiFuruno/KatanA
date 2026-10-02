use crate::font_loader::office_font_leases::DocumentFontLease;
use eframe::egui;
use katana_document_viewer::DocumentGridCell;

const DEFAULT_FONT_SIZE: f32 = 13.0;

pub(super) struct CellFontStyle {
    family: Option<egui::FontFamily>,
    pub(super) faux_bold: bool,
    faux_italic: bool,
}

pub(super) fn style_for_cell(
    ui: &egui::Ui,
    fonts: Option<&DocumentFontLease>,
    cell: &DocumentGridCell,
) -> CellFontStyle {
    let requested = cell.appearance.font_family.trim();
    let bold = cell.appearance.bold;
    let italic = cell.appearance.italic;
    let face = available_family(ui, fonts, requested, bold, italic);
    if face.is_some() {
        return CellFontStyle {
            family: face,
            faux_bold: false,
            faux_italic: false,
        };
    }
    fallback_style(ui, fonts, requested, bold, italic)
}

fn fallback_style(
    ui: &egui::Ui,
    fonts: Option<&DocumentFontLease>,
    requested: &str,
    bold: bool,
    italic: bool,
) -> CellFontStyle {
    let bold_face = bold
        .then(|| available_family(ui, fonts, requested, true, false))
        .flatten();
    if bold_face.is_some() {
        return CellFontStyle {
            family: bold_face,
            faux_bold: false,
            faux_italic: italic,
        };
    }
    let italic_face = italic
        .then(|| available_family(ui, fonts, requested, false, true))
        .flatten();
    if italic_face.is_some() {
        return CellFontStyle {
            family: italic_face,
            faux_bold: bold,
            faux_italic: false,
        };
    }
    let regular_face = available_family(ui, fonts, requested, false, false);
    CellFontStyle {
        family: regular_face,
        faux_bold: bold,
        faux_italic: italic,
    }
}

fn available_family(
    ui: &egui::Ui,
    fonts: Option<&DocumentFontLease>,
    requested: &str,
    bold: bool,
    italic: bool,
) -> Option<egui::FontFamily> {
    let family = fonts?.family_for(requested, bold, italic)?;
    ui.fonts(|fonts| fonts.definitions().families.contains_key(&family))
        .then_some(family)
}

pub(super) fn text_format(
    ui: &egui::Ui,
    cell: &DocumentGridCell,
    color: egui::Color32,
    style: &CellFontStyle,
) -> egui::TextFormat {
    let decoration = egui::Stroke::new(1.0, color);
    egui::TextFormat {
        font_id: font_id(
            ui,
            cell.appearance.font_size_px,
            &cell.appearance.font_family,
            style.family.clone(),
        ),
        color,
        italics: style.faux_italic,
        underline: if cell.appearance.underline {
            decoration
        } else {
            Default::default()
        },
        strikethrough: if cell.appearance.strike {
            decoration
        } else {
            Default::default()
        },
        ..Default::default()
    }
}

fn font_id(
    ui: &egui::Ui,
    size: u16,
    family: &str,
    leased_family: Option<egui::FontFamily>,
) -> egui::FontId {
    let size = if size == 0 {
        DEFAULT_FONT_SIZE
    } else {
        f32::from(size)
    };
    if let Some(family) = leased_family {
        return egui::FontId::new(size, family);
    }
    let family = font_family(ui, family);
    egui::FontId::new(size, family)
}

fn font_family(ui: &egui::Ui, family: &str) -> egui::FontFamily {
    let registered = ui.fonts(|fonts| {
        fonts
            .definitions()
            .families
            .keys()
            .find(|candidate| {
                matches!(candidate, egui::FontFamily::Name(name) if name.eq_ignore_ascii_case(family))
            })
            .cloned()
    });
    if let Some(family) = registered {
        return family;
    }
    let normalized = family.to_ascii_lowercase();
    let monospace = ["mono", "courier", "consolas"]
        .iter()
        .any(|name| normalized.contains(name));
    if monospace {
        egui::FontFamily::Monospace
    } else {
        egui::FontFamily::Proportional
    }
}
