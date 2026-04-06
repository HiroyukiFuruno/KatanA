#![allow(unused_imports)]
#![allow(dead_code)]

mod phases;

use crate::Icon;
use crate::app_state::{AppAction, AppState};
use crate::state::update::UpdatePhase;

const SPACING_MEDIUM: f32 = 8.0;
const SPACING_LARGE: f32 = 12.0;
const MAX_SCROLL_HEIGHT: f32 = 250.0;
const UPDATE_DIALOG_WIDTH: f32 = 600.0;

pub(crate) struct UpdateModal<'a> {
    pub open: &'a mut bool,
    pub state: &'a AppState,
    pub markdown_cache: &'a mut egui_commonmark::CommonMarkCache,
    pub pending_action: &'a mut AppAction,
}

impl<'a> UpdateModal<'a> {
    pub fn new(
        open: &'a mut bool,
        state: &'a AppState,
        markdown_cache: &'a mut egui_commonmark::CommonMarkCache,
        pending_action: &'a mut AppAction,
    ) -> Self {
        Self {
            open,
            state,
            markdown_cache,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        use crate::app_state::UpdatePhase;
        let msgs = &crate::i18n::I18nOps::get().update;
        match &self.state.update.phase {
            Some(UpdatePhase::Downloading { progress }) => {
                phases::show_downloading(ctx, &msgs.title, *progress, &msgs.downloading);
                return;
            }
            Some(UpdatePhase::Installing { progress }) => {
                phases::show_downloading(ctx, &msgs.title, *progress, &msgs.installing);
                return;
            }
            Some(UpdatePhase::ReadyToRelaunch) => {
                let action = phases::show_ready_to_relaunch(
                    ctx,
                    &msgs.title,
                    &msgs.restart_confirm,
                    &msgs.action_restart,
                    &msgs.action_later,
                );
                if let Some(action) = action {
                    *self.pending_action = action;
                    if matches!(self.pending_action, AppAction::DismissUpdate) {
                        *self.open = false;
                    }
                }
                return;
            }
            None => {}
        }

        if self.state.update.checking {
            phases::show_checking(ctx, &msgs.title, &msgs.checking_for_updates);
        } else if let Some(err) = &self.state.update.check_error {
            let color = self.resolve_error_color(ctx);
            let close = phases::show_error(
                ctx,
                &msgs.title,
                err,
                &msgs.failed_to_check,
                &msgs.action_close,
                color,
            );
            if close == Some(true) {
                *self.open = false;
            }
        } else if let Some(latest) = &self.state.update.available {
            self.show_available_dialog(ctx, latest);
        } else {
            show_up_to_date_phase(ctx, msgs, self.open);
        }
    }

    fn resolve_error_color(&self, ctx: &egui::Context) -> egui::Color32 {
        ctx.data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        })
        .map_or(crate::theme_bridge::WHITE, |tc| {
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.error_text)
        })
    }

    fn show_available_dialog(self, ctx: &egui::Context, latest: &katana_core::update::ReleaseInfo) {
        let msgs = crate::i18n::I18nOps::get();
        let msgs = &msgs.update;
        let tag = latest.tag_name.clone();
        let body_text = msgs
            .release_notes_template
            .replace("{version}", tag.as_str())
            .replace("{url}", &latest.html_url);
        let desc = msgs
            .update_available_desc
            .replace("{version}", tag.as_str());
        let markdown_cache = self.markdown_cache;
        let action = crate::widgets::Modal::new("katana_update_dialog_v6", &msgs.title)
            .width(UPDATE_DIALOG_WIDTH)
            .show(
                ctx,
                |ui| Self::render_available_body(ui, &body_text, &desc, markdown_cache),
                |ui| Self::render_available_footer(ui, msgs, &tag),
            );
        if let Some(action) = action {
            *self.pending_action = action;
            if matches!(
                *self.pending_action,
                AppAction::DismissUpdate | AppAction::SkipVersion(_) | AppAction::ShowReleaseNotes
            ) {
                *self.open = false;
            }
        }
    }

    fn render_available_body(
        ui: &mut egui::Ui,
        body_text: &str,
        desc: &str,
        markdown_cache: &mut egui_commonmark::CommonMarkCache,
    ) {
        let msgs = crate::i18n::I18nOps::get();
        ui.label(
            egui::RichText::new(msgs.update.update_available.clone())
                .heading()
                .color(ui.visuals().widgets.active.text_color()),
        );
        ui.add_space(SPACING_MEDIUM);
        ui.label(desc);
        ui.add_space(SPACING_LARGE);
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            egui::ScrollArea::vertical()
                .max_height(MAX_SCROLL_HEIGHT)
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    egui_commonmark::CommonMarkViewer::new()
                        .custom_task_box_fn(Some(
                            &crate::widgets::MarkdownHooksOps::katana_task_box,
                        ))
                        .custom_task_context_menu_fn(Some(
                            &crate::widgets::MarkdownHooksOps::katana_task_context_menu,
                        ))
                        .custom_emoji_fn(Some(
                            &katana_core::emoji::EmojiRasterOps::render_apple_color_emoji_png,
                        ))
                        .show(ui, markdown_cache, body_text);
                });
        });
        ui.add_space(SPACING_LARGE);
    }

    fn render_available_footer(
        ui: &mut egui::Ui,
        msgs: &crate::i18n::UpdateMessages,
        tag: &str,
    ) -> Option<AppAction> {
        phases::show_available_footer(ui, msgs, tag)
    }
}

fn show_up_to_date_phase(ctx: &egui::Context, msgs: &crate::i18n::UpdateMessages, open: &mut bool) {
    let close = phases::show_up_to_date(
        ctx,
        &msgs.title,
        &msgs.up_to_date,
        &msgs.up_to_date_desc,
        &msgs.action_close,
    );
    if close == Some(true) {
        *open = false;
    }
}
