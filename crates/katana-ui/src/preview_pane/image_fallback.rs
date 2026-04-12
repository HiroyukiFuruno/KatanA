use crate::preview_pane::DownloadRequest;
use eframe::egui;

pub struct ImageFallbackOps;

impl ImageFallbackOps {
    pub(crate) fn show_not_installed(
        ui: &mut egui::Ui,
        kind: &str,
        download_url: &str,
        install_path: &std::path::Path,
    ) -> (egui::Rect, Option<DownloadRequest>) {
        let mut request = None;
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
            let path_str = install_path.display().to_string();
            ui.label(
                egui::RichText::new(crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().tool.install_path,
                    &[("path", path_str.as_str())],
                ))
                .small()
                .weak(),
            );
            if crate::icon::IconOps::button_with_icon_str(
                ui,
                &crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().tool.download,
                    &[("tool", kind)],
                ),
            )
            .clicked()
            {
                request = Some(DownloadRequest {
                    url: download_url.to_string(),
                    dest: install_path.to_path_buf(),
                });
            }
        });
        (res.response.rect, request)
    }

    pub(crate) fn show_image_fallback(
        ui: &mut egui::Ui,
        path: &std::path::Path,
    ) -> Option<egui::Rect> {
        let path_str = path.to_string_lossy();
        let is_remote = path_str.starts_with("http://") || path_str.starts_with("https://");
        let i18n = crate::i18n::I18nOps::get();
        let label_text = if is_remote {
            &i18n.preview.remote_image
        } else {
            &i18n.preview.missing_image
        };

        let res = ui.group(|ui| {
            crate::widgets::AlignCenter::new()
                .content(|ui| {
                    let icon = if is_remote {
                        crate::icon::Icon::ExternalLink
                    } else {
                        crate::icon::Icon::Warning
                    };
                    ui.add(icon.ui_image(ui, crate::icon::IconSize::Medium));

                    ui.vertical(|ui| {
                        let color = if is_remote {
                            ui.visuals().text_color()
                        } else {
                            ui.ctx()
                                .data(|d| {
                                    d.get_temp::<katana_platform::theme::ThemeColors>(
                                        egui::Id::new("katana_theme_colors"),
                                    )
                                })
                                .map_or(crate::theme_bridge::WHITE, |tc| {
                                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                        tc.preview.warning_text,
                                    )
                                })
                        };
                        ui.label(egui::RichText::new(label_text).color(color));
                        ui.label(egui::RichText::new(path_str).small().weak());
                    });
                })
                .show(ui);
        });

        res.response.context_menu(|ui| {
            if ui.button(&i18n.action.reveal_in_os).clicked() {
                let mut target_dir = path.to_path_buf();
                if !is_remote
                    && !target_dir.exists()
                    && let Some(parent) = target_dir.parent()
                {
                    target_dir = parent.to_path_buf();
                }
                let _ = open::that(target_dir);
                ui.close_menu();
            }
        });

        Some(res.response.rect)
    }
}
