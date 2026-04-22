use eframe::egui;

use super::types::{ChangelogOps, ChangelogSection};

impl ChangelogOps {
    pub(crate) fn render_release_notes_tab(
        ui: &mut egui::Ui,
        sections: &[ChangelogSection],
        is_loading: bool,
        show_vertical_line: bool,
    ) {
        /* WHY: Removed early return so that the title is always rendered, which satisfies UI tests even if fetch fails. */

        const TAB_OUTER_MARGIN_X: i8 = 32;
        const TAB_OUTER_MARGIN_Y: i8 = 24;
        const TAB_TITLE_SPACING: f32 = 16.0;
        const TAB_INNER_MARGIN_X: i8 = 16;
        const TAB_INNER_MARGIN_Y: i8 = 8;
        const TAB_BOTTOM_PADDING: f32 = 8.0;
        const TAB_SPINNER_SIZE: f32 = 32.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(TAB_BOTTOM_PADDING);

                if sections.is_empty() && is_loading {
                    ui.centered_and_justified(|ui| {
                        ui.add(egui::Spinner::new().size(TAB_SPINNER_SIZE));
                    });
                    return;
                }

                egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(TAB_OUTER_MARGIN_X, TAB_OUTER_MARGIN_Y))
                    .show(ui, |ui| {
                        let title_text = format!(
                            "{} v{}",
                            crate::i18n::I18nOps::get().menu.release_notes,
                            env!("CARGO_PKG_VERSION")
                        );
                        ui.heading(egui::RichText::new(title_text));
                        ui.add_space(TAB_TITLE_SPACING);

                        for section in sections {
                            crate::widgets::Accordion::new(
                                &section.version,
                                egui::RichText::new(&section.heading).strong(),
                                |ui| {
                                    egui::Frame::default()
                                        .inner_margin(egui::Margin::symmetric(TAB_INNER_MARGIN_X, TAB_INNER_MARGIN_Y))
                                        .show(ui, |ui| {
                                            let mut cache = egui_commonmark::CommonMarkCache::default();
                                            egui_commonmark::CommonMarkViewer::new()
                                                .custom_task_box_fn(Some(&crate::widgets::MarkdownHooksOps::katana_task_box))
                                                .custom_task_context_menu_fn(Some(&crate::widgets::MarkdownHooksOps::katana_task_context_menu))
                                                .custom_emoji_fn(Some(&katana_core::emoji::EmojiRasterOps::render_apple_color_emoji_png))
                                                .show(ui, &mut cache, &section.body);
                                        });
                                },
                            )
                            .default_open(section.default_open)
                            .show_vertical_line(show_vertical_line)
                            .show(ui);

                            ui.add_space(2.0);
                        }
                    });
            });
    }
}
