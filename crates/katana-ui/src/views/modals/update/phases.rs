use crate::app_state::AppAction;
use egui::Context;

const SPACING_SMALL: f32 = 4.0;
const SPACING_MEDIUM: f32 = 8.0;
const SPACING_LARGE: f32 = 12.0;
const UPDATE_DIALOG_WIDTH: f32 = 600.0;

pub(super) fn show_downloading(ctx: &Context, title: &str, progress: f32, label: &str) {
    crate::widgets::Modal::new("katana_update_dialog_v6", title)
        .width(UPDATE_DIALOG_WIDTH)
        .show_body_only(ctx, |ui| {
            ui.add_space(SPACING_SMALL);
            ui.add(
                egui::ProgressBar::new(progress)
                    .animate(true)
                    .text(format!("{:.0}%", progress * 100.0)),
            );
            ui.add_space(SPACING_MEDIUM);
            ui.label(label);
        });
}

pub(super) fn show_checking(ctx: &Context, title: &str, label: &str) {
    crate::widgets::Modal::new("katana_update_dialog_v6", title)
        .width(UPDATE_DIALOG_WIDTH)
        .show_body_only(ctx, |ui| {
            ui.add(egui::Spinner::new());
            ui.add_space(SPACING_MEDIUM);
            ui.label(label);
        });
}

pub(super) fn show_ready_to_relaunch(
    ctx: &Context,
    title: &str,
    confirm_text: &str,
    restart_label: &str,
    later_label: &str,
) -> Option<AppAction> {
    crate::widgets::Modal::new("katana_update_dialog_v6", title)
        .width(UPDATE_DIALOG_WIDTH)
        .show(
            ctx,
            |ui| {
                ui.add_space(SPACING_LARGE);
                ui.label(egui::RichText::new(confirm_text).heading());
                ui.add_space(SPACING_LARGE);
            },
            |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            egui::RichText::new(restart_label)
                                .color(ui.visuals().widgets.active.text_color())
                                .strong(),
                        )
                        .clicked()
                    {
                        return Some(AppAction::ConfirmRelaunch);
                    }
                    if ui.button(later_label).clicked() {
                        return Some(AppAction::DismissUpdate);
                    }
                    None
                })
                .inner
            },
        )
}

pub(super) fn show_error(
    ctx: &Context,
    title: &str,
    err: &str,
    failed_label: &str,
    close_label: &str,
    error_color: egui::Color32,
) -> Option<bool> {
    crate::widgets::Modal::new("katana_update_dialog_v6", title)
        .width(UPDATE_DIALOG_WIDTH)
        .show(
            ctx,
            |ui| {
                ui.colored_label(error_color, failed_label);
                ui.add_space(SPACING_SMALL);
                ui.label(err);
            },
            |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(close_label).clicked() {
                        return Some(true);
                    }
                    None
                })
                .inner
            },
        )
}

pub(super) fn show_up_to_date(
    ctx: &Context,
    title: &str,
    heading: &str,
    desc: &str,
    close_label: &str,
) -> Option<bool> {
    crate::widgets::Modal::new("katana_update_dialog_v6", title)
        .width(UPDATE_DIALOG_WIDTH)
        .show(
            ctx,
            |ui| {
                ui.heading(heading);
                ui.add_space(SPACING_SMALL);
                ui.label(desc);
            },
            |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(close_label).clicked() {
                        return Some(true);
                    }
                    None
                })
                .inner
            },
        )
}

pub(super) fn show_available_footer(
    ui: &mut egui::Ui,
    msgs: &crate::i18n::UpdateMessages,
    tag: &str,
) -> Option<crate::app_state::AppAction> {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui
            .button(
                egui::RichText::new(msgs.install_update.clone())
                    .color(ui.visuals().widgets.active.text_color())
                    .strong(),
            )
            .clicked()
        {
            return Some(crate::app_state::AppAction::InstallUpdate);
        }
        if ui.button(crate::i18n::I18nOps::get().menu.release_notes.clone()).clicked() {
            return Some(crate::app_state::AppAction::ShowReleaseNotes);
        }
        if ui.button(msgs.action_skip_version.clone()).clicked() {
            return Some(crate::app_state::AppAction::SkipVersion(tag.to_string()));
        }
        if ui.button(msgs.action_later.clone()).clicked() {
            return Some(crate::app_state::AppAction::DismissUpdate);
        }
        None
    })
    .inner
}
