use eframe::egui;

pub struct ImageFallbackOps;

impl ImageFallbackOps {
    pub(crate) fn show_not_installed(ui: &mut egui::Ui, kind: &str, message: &str) -> egui::Rect {
        let res = ui.group(|ui| {
            ui.label(
                egui::RichText::new(crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().tool.not_installed,
                    &[("tool", kind)],
                ))
                .color(
                    ui.ctx()
                        .data(|d| {
                            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                "katana_theme_colors",
                            ))
                        })
                        .map_or(crate::theme_bridge::WHITE, |tc| {
                            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                tc.preview.warning_text,
                            )
                        }),
                ),
            );
            ui.label(egui::RichText::new(message).small().weak());
        });
        res.response.rect
    }
}
