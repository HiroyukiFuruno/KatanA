use eframe::egui;

const TAB_MAX_WIDTH: f32 = 200.0;

pub(super) fn render_drag_ghost(
    ui: &mut egui::Ui,
    idx: usize,
    ghost_rect: egui::Rect,
    title: &str,
    is_changelog: bool,
    is_active: bool,
    is_pinned: bool,
) {
    egui::Area::new(egui::Id::new("tab_ghost").with(idx))
        .fixed_pos(ghost_rect.min)
        .order(egui::Order::Tooltip)
        .show(ui.ctx(), |ui| {
            ui.set_max_width(TAB_MAX_WIDTH);
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    if is_changelog {
                        ui.add(
                            egui::Button::image_and_text(
                                crate::Icon::Info.ui_image(ui, crate::icon::IconSize::Medium),
                                title,
                            )
                            .selected(is_active)
                            .frame(false),
                        );
                    } else {
                        ui.add(egui::Button::selectable(is_active, title).frame(false));
                    }
                    if is_pinned {
                        ui.add(
                            crate::Icon::Pin
                                .image(crate::icon::IconSize::Small)
                                .tint(ui.visuals().text_color()),
                        );
                    } else {
                        ui.add(crate::Icon::Close.button(ui, crate::icon::IconSize::Small));
                    }
                })
                .show(ui);
        });
}
