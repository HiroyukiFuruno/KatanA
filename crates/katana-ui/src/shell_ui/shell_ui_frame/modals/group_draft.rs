use crate::shell::KatanaApp;
use crate::state::{document::TabGroup, layout::GroupDraft};
use eframe::egui;

const COLORS: [&str; 7] = [
    "#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C",
];

impl KatanaApp {
    pub(super) fn show_group_draft(&mut self, ctx: &egui::Context) {
        let Some(draft) = self.state.layout.group_draft.as_mut() else {
            return;
        };
        let mut result = (false, false);
        egui::Modal::new(egui::Id::new("new_group_draft")).show(ctx, |ui| {
            result = render_form(ui, draft, &self.state.document.tab_groups);
        });
        let (commit, cancel) = result;
        if cancel || commit {
            ctx.data_mut(|data| {
                data.remove_temp::<bool>(egui::Id::new("new_group_name").with("focused"))
            });
        }
        if cancel {
            self.state.layout.group_draft = None;
        } else if commit {
            self.commit_tab_group_draft();
        }
    }
}

fn render_form(ui: &mut egui::Ui, draft: &mut GroupDraft, groups: &[TabGroup]) -> (bool, bool) {
    let messages = crate::i18n::I18nOps::get();
    ui.heading(&messages.tab.create_new_group);
    let response = render_name(ui, &mut draft.name);
    render_palette(ui, &mut draft.color_hex);
    let conflict = crate::state::group_identity::GroupIdentity::conflicts(
        groups,
        None,
        &draft.name,
        &draft.color_hex,
    );
    if conflict {
        ui.colored_label(
            ui.visuals().error_fg_color,
            &messages.tab.group_identity_duplicate,
        );
    }
    let mut commit = false;
    let mut cancel = false;
    crate::widgets::AlignCenter::new()
        .content(|ui| {
            commit = ui.button(&messages.action.confirm).clicked();
            cancel = ui.button(&messages.action.cancel).clicked();
        })
        .show(ui);
    commit |= (response.has_focus() || response.lost_focus())
        && ui.input(|i| i.key_pressed(egui::Key::Enter));
    cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
    (commit && !conflict, cancel)
}

fn render_name(ui: &mut egui::Ui, name: &mut String) -> egui::Response {
    let id = egui::Id::new("new_group_name");
    let response = ui.add(
        egui::TextEdit::singleline(name)
            .id(id)
            .hint_text(&crate::i18n::I18nOps::get().tab.group_name_placeholder),
    );
    if !ui.memory(|m| m.has_focus(id))
        && !ui.data(|d| d.get_temp::<bool>(id.with("focused")).unwrap_or(false))
    {
        response.request_focus();
        ui.data_mut(|d| d.insert_temp(id.with("focused"), true));
    }
    response
}

fn render_palette(ui: &mut egui::Ui, current: &mut String) {
    crate::widgets::AlignCenter::new()
        .content(|ui| {
            for color in COLORS {
                let value = egui::Color32::from_hex(color).expect("palette color");
                let stroke = if current.eq_ignore_ascii_case(color) {
                    egui::Stroke::new(2.0, ui.visuals().text_color())
                } else {
                    egui::Stroke::NONE
                };
                let response = ui.add(egui::Button::new("  ").fill(value).stroke(stroke));
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, color)
                });
                if response.clicked() {
                    *current = color.to_owned();
                }
            }
        })
        .show(ui);
}
