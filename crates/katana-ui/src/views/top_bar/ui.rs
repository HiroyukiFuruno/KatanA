use crate::app_state::{AppAction, ViewMode};
use crate::shell::{
    TAB_DROP_ANIMATION_TIME, TAB_DROP_INDICATOR_WIDTH, TAB_INTER_ITEM_SPACING,
    TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS,
};
use crate::shell_ui::{
    invisible_label, relative_full_path, LIGHT_MODE_ICON_BG, STATUS_BAR_ICON_SPACING,
    STATUS_SUCCESS_GREEN,
};
use eframe::egui;

use super::logic::{compute_drop_points, find_best_drop_index, tab_display_title};

const GROUP_HEADER_CORNER_RADIUS: u8 = 4;
const GROUP_HEADER_PADDING_X: i8 = 6;
const GROUP_HEADER_PADDING_Y: i8 = 3;
const GROUP_HEADER_COLLAPSED_ALPHA: u8 = 20;
const GROUP_HEADER_EXPANDED_ALPHA: u8 = 40;
const GROUP_HEADER_DOT_SIZE: f32 = 8.0;
const GROUP_HEADER_DOT_RADIUS: f32 = 4.0;
const GROUP_HEADER_ITEM_SPACING: f32 = 4.0;
const GROUP_HEADER_FONT_SIZE: f32 = 11.0;

pub(crate) struct StatusBar<'a> {
    pub status: Option<&'a (String, crate::app_state::StatusType)>,
    pub is_dirty: bool,
    pub export_filenames: &'a [String],
}

impl<'a> StatusBar<'a> {
    pub fn new(
        status: Option<&'a (String, crate::app_state::StatusType)>,
        is_dirty: bool,
        export_filenames: &'a [String],
    ) -> Self {
        Self {
            status,
            is_dirty,
            export_filenames,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let export_filenames = self.export_filenames;
        ui.horizontal(|ui| {
            let (msg, kind) = if let Some((msg, kind)) = self.status {
                (msg.as_str(), Some(kind))
            } else {
                (crate::i18n::get().status.ready.as_str(), None)
            };

            let (color, icon) = match kind {
                Some(crate::app_state::StatusType::Error) => (
                    ui.ctx()
                        .data(|d| {
                            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                "katana_theme_colors",
                            ))
                        })
                        .map_or(crate::theme_bridge::WHITE, |tc| {
                            crate::theme_bridge::rgb_to_color32(tc.system.error_text)
                        }),
                    Some(crate::Icon::Error),
                ),
                Some(crate::app_state::StatusType::Warning) => {
                    (ui.visuals().warn_fg_color, Some(crate::Icon::Warning))
                }
                Some(crate::app_state::StatusType::Success) => (
                    crate::theme_bridge::from_rgb(0, STATUS_SUCCESS_GREEN, 0),
                    Some(crate::Icon::Success),
                ),
                Some(crate::app_state::StatusType::Info) => {
                    (ui.visuals().text_color(), Some(crate::Icon::Info))
                }
                _ => (ui.visuals().text_color(), None),
            };

            ui.add_space(STATUS_BAR_ICON_SPACING);
            if let Some(i) = icon {
                ui.add(i.image(crate::icon::IconSize::Medium).tint(color));
                ui.add_space(2.0);
            }
            ui.colored_label(color, msg);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !export_filenames.is_empty() {
                    let total = export_filenames.len();
                    ui.spinner();
                    for (i, filename) in export_filenames.iter().enumerate() {
                        let numbered = crate::i18n::tf(
                            &crate::i18n::get().export.exporting,
                            &[("filename", &format!("({}/{}) {}", i + 1, total, filename))],
                        );
                        ui.label(numbered);
                    }
                }
                const DIRTY_DOT_MAX_HEIGHT: f32 = 10.0;
                if self.is_dirty {
                    ui.add(
                        egui::Image::new(crate::Icon::Dot.uri())
                            .tint(ui.visuals().text_color())
                            .fit_to_exact_size(egui::vec2(
                                DIRTY_DOT_MAX_HEIGHT,
                                DIRTY_DOT_MAX_HEIGHT,
                            )),
                    );
                }
            });
        })
        .response
    }
}

pub(crate) struct TabBar<'a> {
    pub workspace_root: Option<&'a std::path::Path>,
    pub open_documents: &'a [katana_core::document::Document],
    pub active_doc_idx: Option<usize>,
    pub recently_closed_tabs: &'a std::collections::VecDeque<std::path::PathBuf>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
}

impl<'a> TabBar<'a> {
    pub fn new(
        workspace_root: Option<&'a std::path::Path>,
        open_documents: &'a [katana_core::document::Document],
        active_doc_idx: Option<usize>,
        recently_closed_tabs: &'a std::collections::VecDeque<std::path::PathBuf>,
        tab_groups: &'a [crate::state::document::TabGroup],
    ) -> Self {
        Self {
            workspace_root,
            open_documents,
            active_doc_idx,
            recently_closed_tabs,
            tab_groups,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<AppAction> {
        const MAX_TAB_WIDTH: f32 = 200.0;
        const PINNED_TAB_MAX_WIDTH: f32 = 60.0;

        let mut close_idx: Option<usize> = None;
        let mut tab_action: Option<AppAction> = None;
        let mut dragged_source: Option<(usize, f32)> = None;
        let mut tab_rects: Vec<(usize, egui::Rect)> = Vec::new();

        let ws_root = self.workspace_root;
        let doc_count = self.open_documents.len();

        ui.style_mut().interaction.tooltip_delay = TAB_TOOLTIP_SHOW_DELAY_SECS;

        ui.horizontal(|ui| {
            let nav_button_width = TAB_NAV_BUTTONS_AREA_WIDTH;
            let scroll_width = ui.available_width() - nav_button_width;

            let should_scroll = ui.memory_mut(|mem| {
                mem.data
                    .get_temp::<bool>(egui::Id::new("scroll_tab_req"))
                    .unwrap_or(false)
            });

            let scroll_resp = egui::ScrollArea::horizontal()
                .max_width(scroll_width)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .id_salt("tab_scroll")
                .show(ui, |ui| {
                    let mut current_hovered_drop_x = None;
                    let mut dragging_ghost_info = None;

                    let mut grouped_doc_indices = std::collections::HashSet::new();
                    for g in self.tab_groups {
                        for (idx, doc) in self.open_documents.iter().enumerate() {
                            if g.members.contains(&doc.path.to_string_lossy().to_string()) {
                                grouped_doc_indices.insert(idx);
                            }
                        }
                    }

                    enum DrawItem<'a> {
                        GroupHeader(&'a crate::state::document::TabGroup),
                        Tab {
                            idx: usize,
                            group: Option<&'a crate::state::document::TabGroup>,
                        },
                    }

                    let mut draw_items: Vec<DrawItem> = Vec::new();

                    for g in self.tab_groups {
                        draw_items.push(DrawItem::GroupHeader(g));
                        for (idx, doc) in self.open_documents.iter().enumerate() {
                            if g.members.contains(&doc.path.to_string_lossy().to_string()) {
                                draw_items.push(DrawItem::Tab {
                                    idx,
                                    group: Some(g),
                                });
                            }
                        }
                    }

                    for (idx, doc) in self.open_documents.iter().enumerate() {
                        if doc.is_pinned && !grouped_doc_indices.contains(&idx) {
                            draw_items.push(DrawItem::Tab { idx, group: None });
                        }
                    }

                    for (idx, doc) in self.open_documents.iter().enumerate() {
                        if !doc.is_pinned && !grouped_doc_indices.contains(&idx) {
                            draw_items.push(DrawItem::Tab { idx, group: None });
                        }
                    }

                    ui.horizontal(|ui| {
                        for item in draw_items {
                            match item {
                                DrawItem::GroupHeader(g) => {
                                    let base_color = egui::Color32::from_hex(&g.color_hex)
                                        .unwrap_or(ui.visuals().widgets.active.bg_fill);
                                    let text_color = ui.visuals().text_color();

                                    let frame_fill = if g.collapsed {
                                        crate::theme_bridge::from_rgba_unmultiplied(
                                            base_color.r(),
                                            base_color.g(),
                                            base_color.b(),
                                            GROUP_HEADER_COLLAPSED_ALPHA,
                                        )
                                    } else {
                                        crate::theme_bridge::from_rgba_unmultiplied(
                                            base_color.r(),
                                            base_color.g(),
                                            base_color.b(),
                                            GROUP_HEADER_EXPANDED_ALPHA,
                                        )
                                    };
                                    let frame_stroke = egui::Stroke::new(1.0, base_color);

                                    let group_resp = egui::Frame::NONE
                                        .fill(frame_fill)
                                        .stroke(frame_stroke)
                                        .corner_radius(GROUP_HEADER_CORNER_RADIUS)
                                        .inner_margin(egui::Margin::symmetric(
                                            GROUP_HEADER_PADDING_X,
                                            GROUP_HEADER_PADDING_Y,
                                        ))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing.x =
                                                    GROUP_HEADER_ITEM_SPACING;
                                                let (rect, _resp) = ui.allocate_exact_size(
                                                    egui::vec2(
                                                        GROUP_HEADER_DOT_SIZE,
                                                        GROUP_HEADER_DOT_SIZE,
                                                    ),
                                                    egui::Sense::hover(),
                                                );
                                                ui.painter().circle_filled(
                                                    rect.center(),
                                                    GROUP_HEADER_DOT_RADIUS,
                                                    base_color,
                                                );
                                                ui.label(
                                                    egui::RichText::new(&g.name)
                                                        .color(text_color)
                                                        .strong()
                                                        .size(GROUP_HEADER_FONT_SIZE),
                                                );
                                            });
                                        })
                                        .response
                                        .interact(egui::Sense::click())
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                                    if group_resp.clicked() {
                                        tab_action = Some(
                                            crate::app_state::AppAction::ToggleCollapseTabGroup(
                                                g.id.clone(),
                                            ),
                                        );
                                    }

                                    group_resp.context_menu(|ui| {
                                        let i18n = crate::i18n::get();

                                        let mut new_name = g.name.clone();
                                        ui.horizontal(|ui| {
                                            ui.add(egui::TextEdit::singleline(&mut new_name).hint_text(&i18n.tab.group_name_placeholder));
                                        });

                                        const SPACING: f32 = 4.0;
                                        const PALETTE_SIZE: f32 = 16.0;
                                        const PALETTE_RADIUS: f32 = 8.0;
                                        const PALETTE_STROKE: f32 = 2.0;

                                        ui.add_space(SPACING);
                                        let mut new_color = g.color_hex.clone();
                                        ui.horizontal(|ui| {
                                            let colors = ["#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C"];
                                            for c in colors {
                                                let color32 = egui::Color32::from_hex(c).unwrap_or(ui.visuals().text_color());
                                                let (rect, resp) = ui.allocate_exact_size(egui::vec2(PALETTE_SIZE, PALETTE_SIZE), egui::Sense::click());
                                                let fill_color = crate::theme_bridge::from_rgba_unmultiplied(color32.r(), color32.g(), color32.b(), u8::MAX);
                                                ui.painter().circle_filled(rect.center(), PALETTE_RADIUS, fill_color);
                                                if new_color == c {
                                                    ui.painter().circle_stroke(rect.center(), PALETTE_RADIUS, egui::Stroke::new(PALETTE_STROKE, ui.visuals().text_color()));
                                                }
                                                if resp.clicked() {
                                                    new_color = c.to_string();
                                                }
                                            }
                                        });

                                        if new_name != g.name || new_color != g.color_hex {
                                            if new_name != g.name {
                                                tab_action = Some(crate::app_state::AppAction::RenameTabGroup {
                                                    group_id: g.id.clone(),
                                                    new_name: new_name.clone(),
                                                });
                                            }
                                            if new_color != g.color_hex {
                                                tab_action = Some(crate::app_state::AppAction::RecolorTabGroup {
                                                    group_id: g.id.clone(),
                                                    new_color: new_color.clone(),
                                                });
                                            }
                                        }

                                        ui.separator();
                                        if ui.button(&i18n.tab.ungroup).clicked() {
                                            tab_action = Some(crate::app_state::AppAction::UngroupTabGroup(g.id.clone()));
                                            ui.close();
                                        }
                                        if ui.button(&i18n.tab.close_group).clicked() {
                                            tab_action = Some(crate::app_state::AppAction::CloseTabGroup(g.id.clone()));
                                            ui.close();
                                        }
                                    });
                                }
                                DrawItem::Tab { idx, group } => {
                                    let doc = &self.open_documents[idx];
                                    let is_active = self.active_doc_idx == Some(idx);

                                    if let Some(g) = group {
                                        if g.collapsed && !is_active {
                                            continue;
                                        }
                                    }

                                    let original_filename = doc.file_name().unwrap_or("untitled");
                                    let is_changelog = doc
                                        .path
                                        .to_string_lossy()
                                        .starts_with("Katana://ChangeLog");

                                    let title = tab_display_title(
                                        original_filename,
                                        is_changelog,
                                        doc.is_dirty,
                                        doc.is_pinned,
                                    );
                                    let tooltip_path = relative_full_path(&doc.path, ws_root);

                                    let (title_resp, close_resp) = ui
                                        .push_id(format!("tab_{idx}"), |ui| {
                                            ui.set_max_width(if doc.is_pinned {
                                                PINNED_TAB_MAX_WIDTH
                                            } else {
                                                MAX_TAB_WIDTH
                                            });
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Truncate);

                                            let t_resp = if is_changelog {
                                                ui.add(
                                                    egui::Button::image_and_text(
                                                        crate::Icon::Info.ui_image(
                                                            ui,
                                                            crate::icon::IconSize::Small,
                                                        ),
                                                        &title,
                                                    )
                                                    .selected(is_active),
                                                )
                                            } else {
                                                ui.add(egui::Button::selectable(is_active, &title))
                                            };

                                            let c_resp =
                                                if !doc.is_pinned {
                                                    Some(ui.add(egui::Button::image_and_text(
                                                        crate::Icon::Close.ui_image(
                                                            ui,
                                                            crate::icon::IconSize::Small,
                                                        ),
                                                        invisible_label("x"),
                                                    )))
                                                } else {
                                                    None
                                                };
                                            (t_resp, c_resp)
                                        })
                                        .inner;

                                    let full_tab_rect = if let Some(c) = &close_resp {
                                        title_resp.rect.union(c.rect)
                                    } else {
                                        title_resp.rect
                                    };
                                    tab_rects.push((idx, full_tab_rect));

                                    if let Some(g) = group {
                                        let base_color = egui::Color32::from_hex(&g.color_hex)
                                            .unwrap_or(ui.visuals().widgets.active.bg_fill);
                                        let line_y = full_tab_rect.bottom() - 1.0;
                                        ui.painter().hline(
                                            full_tab_rect.x_range(),
                                            line_y,
                                            egui::Stroke::new(2.0, base_color),
                                        );
                                    }

                                    let tab_interact = ui.interact(
                                        title_resp.rect,
                                        egui::Id::new("tab_interact").with(idx),
                                        egui::Sense::click_and_drag(),
                                    );

                                    let mut clicked_tab = tab_interact.clicked();
                                    if let Some(c) = close_resp {
                                        if c.clicked() {
                                            close_idx = Some(idx);
                                            clicked_tab = false;
                                        }
                                    }

                                    let is_being_dragged =
                                        ui.ctx().is_being_dragged(tab_interact.id);
                                    if is_being_dragged {
                                        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                            let press_origin = ui
                                                .input(|i| i.pointer.press_origin())
                                                .unwrap_or(pointer_pos);
                                            let drag_offset = pointer_pos - press_origin;
                                            let ghost_rect = full_tab_rect.translate(drag_offset);

                                            ui.memory_mut(|mem| {
                                                mem.data.insert_temp(
                                                    egui::Id::new("drag_ghost_x").with(idx),
                                                    ghost_rect.center().x,
                                                )
                                            });

                                            ui.scroll_to_rect(ghost_rect, None);

                                            egui::Area::new(egui::Id::new("tab_ghost").with(idx))
                                                .fixed_pos(ghost_rect.min)
                                                .order(egui::Order::Tooltip)
                                                .show(ui.ctx(), |ui| {
                                                    ui.set_max_width(if doc.is_pinned {
                                                        PINNED_TAB_MAX_WIDTH
                                                    } else {
                                                        MAX_TAB_WIDTH
                                                    });
                                                    ui.style_mut().wrap_mode =
                                                        Some(egui::TextWrapMode::Truncate);

                                                    ui.horizontal(|ui| {
                                                        ui.spacing_mut().item_spacing.x = 0.0;
                                                        if is_changelog {
                                                            let btn = egui::Button::image_and_text(
                                                                crate::Icon::Info.ui_image(
                                                                    ui,
                                                                    crate::icon::IconSize::Medium,
                                                                ),
                                                                &title,
                                                            )
                                                            .selected(is_active);
                                                            ui.add(btn);
                                                        } else {
                                                            ui.add(egui::Button::selectable(
                                                                is_active, &title,
                                                            ));
                                                        }
                                                        if !doc.is_pinned {
                                                            ui.add(egui::Button::image_and_text(
                                                                crate::Icon::Close.ui_image(
                                                                    ui,
                                                                    crate::icon::IconSize::Small,
                                                                ),
                                                                invisible_label("x"),
                                                            ));
                                                        }
                                                    });
                                                });

                                            dragging_ghost_info =
                                                Some((ghost_rect, full_tab_rect.y_range()));
                                        }
                                    }

                                    if is_active && should_scroll {
                                        tab_interact.scroll_to_me(Some(egui::Align::Center));
                                    }

                                    if tab_interact.drag_stopped() {
                                        if let Some(ghost_x) = ui.memory(|mem| {
                                            mem.data.get_temp::<f32>(
                                                egui::Id::new("drag_ghost_x").with(idx),
                                            )
                                        }) {
                                            dragged_source = Some((idx, ghost_x));
                                        }
                                    }

                                    let tab_interact = tab_interact.on_hover_text(&tooltip_path);

                                    tab_interact.context_menu(|ui| {
                                        let i18n = crate::i18n::get();

                                        if ui.button(&i18n.tab.close).clicked() {
                                            tab_action = Some(AppAction::CloseDocument(idx));
                                            ui.close();
                                        }
                                        if ui.button(&i18n.tab.close_others).clicked() {
                                            tab_action = Some(AppAction::CloseOtherDocuments(idx));
                                            ui.close();
                                        }
                                        if ui.button(&i18n.tab.close_all).clicked() {
                                            tab_action = Some(AppAction::CloseAllDocuments);
                                            ui.close();
                                        }
                                        if ui.button(&i18n.tab.close_right).clicked() {
                                            tab_action =
                                                Some(AppAction::CloseDocumentsToRight(idx));
                                            ui.close();
                                        }
                                        if ui.button(&i18n.tab.close_left).clicked() {
                                            tab_action = Some(AppAction::CloseDocumentsToLeft(idx));
                                            ui.close();
                                        }
                                        ui.separator();
                                        let pin_label = if doc.is_pinned {
                                            &i18n.tab.unpin
                                        } else {
                                            &i18n.tab.pin
                                        };
                                        if ui.button(pin_label).clicked() {
                                            tab_action = Some(AppAction::TogglePinDocument(idx));
                                            ui.close();
                                        }

                                        if !doc.is_pinned {
                                            ui.menu_button(&i18n.tab.create_new_group, |ui| {
                                                let mut temp_name = ui.memory_mut(|mem| {
                                                    mem.data.get_temp::<String>(egui::Id::new("tmp_group_name").with(idx))
                                                }).unwrap_or_else(String::new);
                                                let mut temp_color = ui.memory_mut(|mem| {
                                                    mem.data.get_temp::<String>(egui::Id::new("tmp_group_color").with(idx))
                                                }).unwrap_or_else(|| "#4A90D9".to_string());

                                                ui.horizontal(|ui| {
                                                    ui.add(egui::TextEdit::singleline(&mut temp_name)
                                                        .hint_text(&i18n.tab.group_name_placeholder));
                                                });

                                                const SPACING: f32 = 4.0;
                                                const PALETTE_SIZE: f32 = 16.0;
                                                const PALETTE_RADIUS: f32 = 8.0;
                                                const PALETTE_STROKE: f32 = 2.0;

                                                ui.add_space(SPACING);
                                                ui.horizontal(|ui| {
                                                    let colors = ["#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C"];
                                                    for c in colors {
                                                        let color32 = egui::Color32::from_hex(c).unwrap_or(ui.visuals().text_color());
                                                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(PALETTE_SIZE, PALETTE_SIZE), egui::Sense::click());
                                                        // Calculate alpha to respect linter (though test is disabled)
                                                        let fill_color = crate::theme_bridge::from_rgba_unmultiplied(
                                                            color32.r(), color32.g(), color32.b(), u8::MAX
                                                        );
                                                        ui.painter().circle_filled(rect.center(), PALETTE_RADIUS, fill_color);
                                                        if temp_color == c {
                                                            ui.painter().circle_stroke(rect.center(), PALETTE_RADIUS, egui::Stroke::new(PALETTE_STROKE, ui.visuals().text_color()));
                                                        }
                                                        if resp.clicked() {
                                                            temp_color = c.to_string();
                                                        }
                                                    }
                                                });
                                                ui.add_space(SPACING);

                                                ui.add_enabled_ui(!temp_name.trim().is_empty(), |ui| {
                                                    if ui.button(&i18n.tab.create_group_button).clicked() {
                                                        tab_action = Some(AppAction::CreateTabGroup {
                                                            name: temp_name.trim().to_string(),
                                                            color_hex: temp_color.clone(),
                                                            initial_member: doc.path.clone(),
                                                        });
                                                        ui.memory_mut(|mem| {
                                                            mem.data.remove::<String>(egui::Id::new("tmp_group_name").with(idx));
                                                            mem.data.remove::<String>(egui::Id::new("tmp_group_color").with(idx));
                                                        });
                                                        ui.close();
                                                    }
                                                });

                                                ui.memory_mut(|mem| {
                                                    mem.data.insert_temp(egui::Id::new("tmp_group_name").with(idx), temp_name);
                                                    mem.data.insert_temp(egui::Id::new("tmp_group_color").with(idx), temp_color);
                                                });
                                            });

                                            let mut is_in_any_group = false;
                                            let doc_str = doc.path.to_string_lossy().to_string();
                                            for g in self.tab_groups {
                                                if g.members.contains(&doc_str) {
                                                    is_in_any_group = true;
                                                    break;
                                                }
                                            }

                                            if !self.tab_groups.is_empty() {
                                                ui.menu_button(&i18n.tab.tab_group, |ui| {
                                                    for g in self.tab_groups {
                                                        let is_member = g.members.contains(&doc_str);
                                                        let label = if is_member {
                                                            i18n.tab.added_to_group.replace("{group_name}", &g.name)
                                                        } else {
                                                            i18n.tab.add_to_group.replace("{group_name}", &g.name)
                                                        };
                                                        if ui.radio(is_member, label).clicked() {
                                                            tab_action = Some(AppAction::AddTabToGroup {
                                                                group_id: g.id.clone(),
                                                                member: doc.path.clone(),
                                                            });
                                                            ui.close();
                                                        }
                                                    }
                                                });
                                            }

                                            if is_in_any_group {
                                                ui.separator();
                                                if ui.button(&i18n.tab.remove_from_group).clicked() {
                                                    tab_action = Some(AppAction::RemoveTabFromGroup(doc.path.clone()));
                                                    ui.close();
                                                }
                                            }
                                        }

                                        if !self.recently_closed_tabs.is_empty() {
                                            ui.separator();
                                            if ui.button(&i18n.tab.restore_closed).clicked() {
                                                tab_action = Some(AppAction::RestoreClosedDocument);
                                                ui.close();
                                            }
                                        }
                                    });

                                    if clicked_tab && !is_active {
                                        tab_action =
                                            Some(AppAction::SelectDocument(doc.path.clone()));
                                    }

                                    ui.add_space(TAB_INTER_ITEM_SPACING);
                                }
                            }
                        }

                        let drop_points = compute_drop_points(&tab_rects);

                        if let Some((ghost_rect, y_range)) = dragging_ghost_info {
                            let mut best_dist = f32::MAX;
                            let mut best_x = None;
                            for (_insert_idx, x) in &drop_points {
                                let dist = (ghost_rect.center().x - x).abs();
                                if dist < best_dist {
                                    best_dist = dist;
                                    best_x = Some(*x);
                                }
                            }
                            if let Some(x) = best_x {
                                current_hovered_drop_x = Some((x, y_range));
                            }
                        }

                        if let Some((target_x, y_range)) = current_hovered_drop_x {
                            let indicator_id = egui::Id::new("tab_drop_indicator");
                            let animated_x = ui.ctx().animate_value_with_time(
                                indicator_id,
                                target_x,
                                TAB_DROP_ANIMATION_TIME,
                            );

                            let stroke = egui::Stroke::new(
                                TAB_DROP_INDICATOR_WIDTH,
                                ui.visuals().selection.bg_fill,
                            );
                            ui.painter().vline(animated_x, y_range, stroke);
                        }
                    });
                });

            if should_scroll {
                ui.memory_mut(|mem| {
                    mem.data
                        .remove_temp::<bool>(egui::Id::new("scroll_tab_req"));
                });
            }

            ui.separator();

            let nav_enabled = doc_count > 1;
            let icon_bg = if ui.visuals().dark_mode {
                crate::theme_bridge::TRANSPARENT
            } else {
                crate::theme_bridge::from_gray(LIGHT_MODE_ICON_BG)
            };

            if ui
                .add_enabled(
                    nav_enabled,
                    egui::Button::image_and_text(
                        crate::Icon::TriangleLeft.ui_image(ui, crate::icon::IconSize::Small),
                        invisible_label("◀"),
                    )
                    .fill(icon_bg),
                )
                .on_hover_text(crate::i18n::get().tab.nav_prev.clone())
                .clicked()
            {
                if let Some(idx) = self.active_doc_idx {
                    let new_idx = crate::shell_logic::prev_tab_index(idx, doc_count);
                    tab_action = Some(AppAction::SelectDocument(
                        self.open_documents[new_idx].path.clone(),
                    ));
                    ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
                }
            }
            if ui
                .add_enabled(
                    nav_enabled,
                    egui::Button::image_and_text(
                        crate::Icon::TriangleRight.ui_image(ui, crate::icon::IconSize::Small),
                        invisible_label("▶"),
                    )
                    .fill(icon_bg),
                )
                .on_hover_text(crate::i18n::get().tab.nav_next.clone())
                .clicked()
            {
                if let Some(idx) = self.active_doc_idx {
                    let new_idx = crate::shell_logic::next_tab_index(idx, doc_count);
                    tab_action = Some(AppAction::SelectDocument(
                        self.open_documents[new_idx].path.clone(),
                    ));
                    ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
                }
            }

            scroll_resp.inner
        });

        if let Some((src_idx, ghost_center_x)) = dragged_source {
            let drop_points = compute_drop_points(&tab_rects);
            if let Some(to_visual) = find_best_drop_index(&drop_points, ghost_center_x) {
                let to_physical = if to_visual < tab_rects.len() {
                    tab_rects[to_visual].0 // insert before physical element at to_visual
                } else {
                    self.open_documents.len()
                };

                let mut new_group_id = Some(None); // Default is ungrouped

                if to_visual < tab_rects.len() {
                    let neighbor_idx = tab_rects[to_visual].0;
                    if let Some(doc) = self.open_documents.get(neighbor_idx) {
                        let path_str = doc.path.to_string_lossy().to_string();
                        for g in self.tab_groups {
                            if g.members.contains(&path_str) {
                                new_group_id = Some(Some(g.id.clone()));
                                break;
                            }
                        }
                    }
                } else if to_visual > 0 {
                    // check the last element if we insert at the end
                    let neighbor_idx = tab_rects[to_visual - 1].0;
                    if let Some(doc) = self.open_documents.get(neighbor_idx) {
                        let path_str = doc.path.to_string_lossy().to_string();
                        for g in self.tab_groups {
                            if g.members.contains(&path_str) {
                                new_group_id = Some(Some(g.id.clone()));
                                break;
                            }
                        }
                    }
                }

                if src_idx != to_physical && src_idx + 1 != to_physical {
                    tab_action = Some(AppAction::ReorderDocument {
                        from: src_idx,
                        to: to_physical,
                        new_group_id,
                    });
                } else if let Some(Some(ref g_id)) = new_group_id {
                    let path_str = self.open_documents[src_idx]
                        .path
                        .to_string_lossy()
                        .to_string();
                    let is_in_group = self
                        .tab_groups
                        .iter()
                        .find(|g| g.id == *g_id)
                        .is_some_and(|g| g.members.contains(&path_str));
                    // Even if physical order doesn't change, we might still be dragged into a group
                    if !is_in_group {
                        tab_action = Some(AppAction::ReorderDocument {
                            from: src_idx,
                            to: src_idx, // same physical place, just join group
                            new_group_id,
                        });
                    }
                } else if let Some(None) = new_group_id {
                    let path_str = self.open_documents[src_idx]
                        .path
                        .to_string_lossy()
                        .to_string();
                    let is_in_any = self
                        .tab_groups
                        .iter()
                        .any(|g| g.members.contains(&path_str));
                    if is_in_any {
                        tab_action = Some(AppAction::ReorderDocument {
                            from: src_idx,
                            to: src_idx,
                            new_group_id,
                        });
                    }
                }
            }
        }

        if let Some(idx) = close_idx {
            tab_action = Some(AppAction::CloseDocument(idx));
        }

        tab_action
    }
}

pub(crate) struct ViewModeBar {
    pub view_mode: ViewMode,
    pub is_changelog: bool,
    pub split_direction: katana_platform::SplitDirection,
    pub pane_order: katana_platform::PaneOrder,
    pub scroll_sync_enabled: bool,
    pub scroll_sync_override: Option<bool>,
    pub update_available: bool,
    pub update_checking: bool,
}

impl ViewModeBar {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        view_mode: ViewMode,
        is_changelog: bool,
        split_direction: katana_platform::SplitDirection,
        pane_order: katana_platform::PaneOrder,
        scroll_sync_enabled: bool,
        scroll_sync_override: Option<bool>,
        update_available: bool,
        update_checking: bool,
    ) -> Self {
        Self {
            view_mode,
            is_changelog,
            split_direction,
            pane_order,
            scroll_sync_enabled,
            scroll_sync_override,
            update_available,
            update_checking,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<AppAction> {
        let mut action: Option<AppAction> = None;
        let mut mode = self.view_mode;
        let prev = mode;
        let bar_height = ui.spacing().interact_size.y;
        let available_width = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(available_width, bar_height),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if self.update_available && !self.update_checking {
                    const COLOR_SUCCESS_G: u8 = 200;
                    let badge_str = format!("✨ {}", crate::i18n::get().update.update_available);
                    let badge_text = egui::RichText::new(badge_str)
                        .color(crate::theme_bridge::from_rgb(0, COLOR_SUCCESS_G, 100))
                        .strong();

                    if ui
                        .add(egui::Button::new(badge_text).sense(egui::Sense::click()))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(AppAction::CheckForUpdates);
                    }
                    ui.separator();
                }

                let is_changelog = self.is_changelog;

                let prev_is_split = prev == ViewMode::Split;
                let is_split = mode == ViewMode::Split;

                if !is_changelog {
                    if ui
                        .add(egui::Button::image_and_text(
                            crate::Icon::Refresh.ui_image(ui, crate::icon::IconSize::Medium),
                            invisible_label("🔄"),
                        ))
                        .on_hover_text(crate::i18n::get().action.refresh_document.clone())
                        .clicked()
                    {
                        action = Some(AppAction::RefreshDocument { is_manual: true });
                    }
                    ui.separator();

                    if ui
                        .selectable_label(is_split, crate::i18n::get().view_mode.split.clone())
                        .clicked()
                        && !is_split
                    {
                        mode = ViewMode::Split;
                    }

                    ui.selectable_value(
                        &mut mode,
                        ViewMode::CodeOnly,
                        crate::i18n::get().view_mode.code.clone(),
                    );
                    ui.selectable_value(
                        &mut mode,
                        ViewMode::PreviewOnly,
                        crate::i18n::get().view_mode.preview.clone(),
                    );
                }

                if !is_changelog && is_split && (is_split == prev_is_split) {
                    ui.separator();

                    let current_dir = self.split_direction;
                    let (dir_icon, dir_tip) = match current_dir {
                        katana_platform::SplitDirection::Horizontal => (
                            crate::icon::Icon::SplitHorizontal,
                            crate::i18n::get().split_toggle.vertical.clone(),
                        ),
                        katana_platform::SplitDirection::Vertical => (
                            crate::icon::Icon::SplitVertical,
                            crate::i18n::get().split_toggle.horizontal.clone(),
                        ),
                    };
                    let icon_size = crate::icon::IconSize::Medium;
                    let resp_dir = ui
                        .add(egui::Button::image(
                            dir_icon.image(icon_size).tint(ui.visuals().text_color()),
                        ))
                        .on_hover_text(dir_tip);

                    resp_dir.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            "Toggle Split Direction",
                        )
                    });

                    if resp_dir.clicked() {
                        let new_dir = match current_dir {
                            katana_platform::SplitDirection::Horizontal => {
                                katana_platform::SplitDirection::Vertical
                            }
                            katana_platform::SplitDirection::Vertical => {
                                katana_platform::SplitDirection::Horizontal
                            }
                        };
                        action = Some(AppAction::SetSplitDirection(new_dir));
                    }

                    let current_order = self.pane_order;
                    let (order_text, order_tip) = match current_order {
                        katana_platform::PaneOrder::EditorFirst => (
                            "📄|👁",
                            crate::i18n::get().split_toggle.preview_first.clone(),
                        ),
                        katana_platform::PaneOrder::PreviewFirst => {
                            ("👁|📄", crate::i18n::get().split_toggle.editor_first.clone())
                        }
                    };
                    if ui
                        .add(egui::Button::new(order_text).sense(egui::Sense::click()))
                        .on_hover_text(order_tip)
                        .clicked()
                    {
                        let new_order = match current_order {
                            katana_platform::PaneOrder::EditorFirst => {
                                katana_platform::PaneOrder::PreviewFirst
                            }
                            katana_platform::PaneOrder::PreviewFirst => {
                                katana_platform::PaneOrder::EditorFirst
                            }
                        };
                        action = Some(AppAction::SetPaneOrder(new_order));
                    }

                    ui.separator();

                    let mut is_on = self
                        .scroll_sync_override
                        .unwrap_or(self.scroll_sync_enabled);

                    const TOGGLE_LABEL_SPACING: f32 = 8.0;

                    let toggle_resp = crate::widgets::toggle_switch(ui, &mut is_on);
                    ui.add_space(TOGGLE_LABEL_SPACING);
                    let text_resp = ui.selectable_label(
                        false,
                        crate::i18n::get().settings.behavior.scroll_sync.clone(),
                    );

                    let toggled = text_resp.clicked() || toggle_resp.clicked();
                    if text_resp.clicked() && !toggle_resp.clicked() {
                        is_on = !is_on;
                    }

                    if toggled {
                        action = Some(AppAction::ToggleScrollSync(is_on));
                    }
                }
            },
        );
        if mode != prev {
            action = Some(AppAction::SetViewMode(mode));
        }

        action
    }
}
