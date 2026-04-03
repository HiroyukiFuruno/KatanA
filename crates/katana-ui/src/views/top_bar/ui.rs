use crate::app_state::{AppAction, ViewMode};
use crate::shell::{
    TAB_DROP_ANIMATION_TIME, TAB_DROP_INDICATOR_WIDTH, TAB_INTER_ITEM_SPACING,
    TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS,
};
use crate::shell_ui::{
    LIGHT_MODE_ICON_BG, STATUS_BAR_ICON_SPACING, STATUS_SUCCESS_GREEN, invisible_label,
    relative_full_path,
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

const GROUP_POPUP_MIN_WIDTH: f32 = 200.0;
const GROUP_MENU_ICON_SIZE: f32 = 12.0;
const GROUP_MENU_ICON_RADIUS: f32 = 4.0;

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

    pub fn show(self, ui: &mut egui::Ui, problem_count: usize) -> Option<AppAction> {
        let export_filenames = self.export_filenames;
        let mut action = None;
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
            crate::icon::render_str_with_icons(ui, msg, Some(color));

            let problem_text = crate::i18n::tf(
                &crate::i18n::get().status.problems_count_format,
                &[("count", &problem_count.to_string())],
            );
            let btn = egui::Button::image_and_text(
                crate::Icon::Warning.ui_image(ui, crate::icon::IconSize::Small),
                problem_text,
            );
            if ui
                .add(btn)
                .on_hover_text(crate::i18n::get().status.toggle_problems_panel.clone())
                .clicked()
            {
                action = Some(AppAction::ToggleProblemsPanel);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !export_filenames.is_empty() {
                    let total = export_filenames.len();
                    ui.spinner();
                    for (i, filename) in export_filenames.iter().enumerate() {
                        let numbered = crate::i18n::tf(
                            &crate::i18n::get().export.exporting,
                            &[("filename", &format!("({}/{}) {}", i + 1, total, filename))],
                        );
                        crate::icon::render_str_with_icons(ui, &numbered, None);
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
        });
        action
    }
}

pub(crate) struct TabBar<'a> {
    pub workspace_root: Option<&'a std::path::Path>,
    pub open_documents: &'a [katana_core::document::Document],
    pub active_doc_idx: Option<usize>,
    pub recently_closed_tabs: &'a std::collections::VecDeque<(std::path::PathBuf, bool)>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
    pub inline_rename_group: &'a Option<String>,
}

impl<'a> TabBar<'a> {
    pub fn new(
        workspace_root: Option<&'a std::path::Path>,
        open_documents: &'a [katana_core::document::Document],
        active_doc_idx: Option<usize>,
        recently_closed_tabs: &'a std::collections::VecDeque<(std::path::PathBuf, bool)>,
        tab_groups: &'a [crate::state::document::TabGroup],
        inline_rename_group: &'a Option<String>,
    ) -> Self {
        Self {
            workspace_root,
            open_documents,
            active_doc_idx,
            recently_closed_tabs,
            tab_groups,
            inline_rename_group,
        }
    }

    #[allow(deprecated)]
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

                    for (idx, doc) in self.open_documents.iter().enumerate() {
                        if doc.is_pinned {
                            draw_items.push(DrawItem::Tab { idx, group: None });
                        }
                    }

                    for g in self.tab_groups {
                        draw_items.push(DrawItem::GroupHeader(g));
                        for (idx, doc) in self.open_documents.iter().enumerate() {
                            if !doc.is_pinned && g.members.contains(&doc.path.to_string_lossy().to_string()) {
                                draw_items.push(DrawItem::Tab {
                                    idx,
                                    group: Some(g),
                                });
                            }
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

                                    if group_resp.secondary_clicked() {
                                        ui.memory_mut(|mem: &mut egui::Memory| { #[allow(deprecated)] { mem.toggle_popup(egui::Id::new("group_popup").with(&g.id)); } });
                                    }

                                    let popup_id = egui::Id::new("group_popup").with(&g.id);
                                    if self.inline_rename_group.as_ref() == Some(&g.id) {
                                        ui.memory_mut(|mem: &mut egui::Memory| { #[allow(deprecated)] { mem.open_popup(popup_id); } });
                                        tab_action = Some(crate::app_state::AppAction::ClearInlineRename);
                                    }

                                    let popup_resp = egui::popup_below_widget(ui, popup_id, &group_resp, egui::PopupCloseBehavior::IgnoreClicks, |ui: &mut egui::Ui| {
                                        ui.set_min_width(GROUP_POPUP_MIN_WIDTH);
                                        let i18n = crate::i18n::get();
                                        let mut new_name = g.name.clone();
                                        let mut new_color = g.color_hex.clone();
                                        ui.horizontal(|ui: &mut egui::Ui| {
                                            let resp = ui.add(egui::TextEdit::singleline(&mut new_name).hint_text(&i18n.tab.group_name_placeholder));
                                            if self.inline_rename_group.as_ref() == Some(&g.id) {
                                                resp.request_focus();
                                            }
                                        });

                                        const SPACING: f32 = 4.0;
                                        const PALETTE_SIZE: f32 = 16.0;
                                        const PALETTE_RADIUS: f32 = 8.0;
                                        const PALETTE_STROKE: f32 = 2.0;

                                        ui.add_space(SPACING);
                                        ui.horizontal(|ui: &mut egui::Ui| {
                                            let colors = ["#4A90D9", "#D94A4A", "#4AD97A", "#D9A04A", "#9B59B6", "#F1C40F", "#1ABC9C"];
                                            for c in colors {
                                                let color32 = egui::Color32::from_hex(c).unwrap_or_default();
                                                let (rect, resp) = ui.allocate_exact_size(egui::vec2(PALETTE_SIZE, PALETTE_SIZE), egui::Sense::click());
                                                ui.painter().circle_filled(rect.center(), PALETTE_RADIUS, color32);
                                                if new_color == c {
                                                    ui.painter().circle_stroke(rect.center(), PALETTE_RADIUS, egui::Stroke::new(PALETTE_STROKE, ui.visuals().text_color()));
                                                }
                                                if resp.clicked() {
                                                    new_color = c.to_string();
                                                }
                                            }
                                        });
                                        ui.add_space(SPACING);

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
                                            ui.memory_mut(|mem| { #[allow(deprecated)] { mem.close_popup(popup_id); } });
                                        }
                                        if ui.button(&i18n.tab.close_group).clicked() {
                                            tab_action = Some(crate::app_state::AppAction::CloseTabGroup(g.id.clone()));
                                            ui.memory_mut(|mem| { #[allow(deprecated)] { mem.close_popup(popup_id); } });
                                        }

                                        ui.min_rect()
                                    });

                                    // Manually implement CloseOnClickOutside to avoid 'focusing text field closes popup' bug in older egui
                                    if ui.memory(|mem| mem.is_popup_open(popup_id))
                                        && ui.input(|i| i.pointer.any_pressed() && i.pointer.primary_pressed())
                                        && let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                            let click_outside_popup = popup_resp.is_none_or(|r| !r.contains(pos));
                                            let click_outside_header = !group_resp.rect.contains(pos);
                                            if click_outside_popup && click_outside_header {
                                                ui.memory_mut(|mem| { #[allow(deprecated)] { mem.close_popup(popup_id); } });
                                            }
                                        }
                                }
                                DrawItem::Tab { idx, group } => {
                                    let doc = &self.open_documents[idx];
                                    let is_active = self.active_doc_idx == Some(idx);

                                    if let Some(g) = group
                                        && g.collapsed {
                                            continue;
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
                                            } else if doc.is_pinned {
                                                ui.add(
                                                    egui::Button::image_and_text(
                                                        crate::Icon::Pin.ui_image(
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
                                    if let Some(c) = close_resp
                                        && c.clicked() {
                                            close_idx = Some(idx);
                                            clicked_tab = false;
                                        }

                                    let is_being_dragged =
                                        ui.ctx().is_being_dragged(tab_interact.id);
                                    if is_being_dragged
                                        && let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
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

                                    if is_active && should_scroll {
                                        tab_interact.scroll_to_me(Some(egui::Align::Center));
                                    }

                                    if tab_interact.drag_stopped()
                                        && let Some(ghost_x) = ui.memory(|mem| {
                                            mem.data.get_temp::<f32>(
                                                egui::Id::new("drag_ghost_x").with(idx),
                                            )
                                        }) {
                                            dragged_source = Some((idx, ghost_x));
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
                                            let mut is_in_any_group = false;
                                            let doc_str = doc.path.to_string_lossy().to_string();
                                            for g in self.tab_groups {
                                                if g.members.contains(&doc_str) {
                                                    is_in_any_group = true;
                                                    break;
                                                }
                                            }

                                            // Chrome logic:
                                            // Top-level: "Add tab to new group" (if not in any group)
                                            // If in a group: "Remove from group"
                                            if !is_in_any_group {
                                                if ui.button(&i18n.tab.create_new_group).clicked() {
                                                    tab_action = Some(AppAction::CreateTabGroup {
                                                        name: String::new(),
                                                        color_hex: "#4A90D9".to_string(),
                                                        initial_member: doc.path.clone(),
                                                    });
                                                    ui.close();
                                                }
                                            } else {
                                                if ui.button(&i18n.tab.remove_from_group).clicked() {
                                                    tab_action = Some(AppAction::RemoveTabFromGroup(doc.path.clone()));
                                                    ui.close();
                                                }
                                            }

                                            // Submenu "Add to group" should appear if there's any other group OR if it's already in a group (so they can move to a 'New group').
                                            let has_other_groups = self.tab_groups.iter().any(|g| !g.members.contains(&doc_str));
                                            if has_other_groups || is_in_any_group {
                                                ui.menu_button(&i18n.tab.add_to_group, |ui| {
                                                    if is_in_any_group {
                                                        if ui.button(&i18n.tab.create_new_group).clicked() {
                                                            tab_action = Some(AppAction::CreateTabGroup {
                                                                name: String::new(),
                                                                color_hex: "#4A90D9".to_string(),
                                                                initial_member: doc.path.clone(),
                                                            });
                                                            ui.close();
                                                        }
                                                        ui.separator();
                                                    }

                                                    for g in self.tab_groups {
                                                        if g.members.contains(&doc_str) {
                                                            continue;
                                                        }
                                                        ui.horizontal(|ui: &mut egui::Ui| {
                                                            let color32 = egui::Color32::from_hex(&g.color_hex).unwrap_or(ui.visuals().widgets.active.bg_fill);
                                                            let (rect, _) = ui.allocate_exact_size(egui::vec2(GROUP_MENU_ICON_SIZE, GROUP_MENU_ICON_SIZE), egui::Sense::hover());
                                                            ui.painter().circle_filled(rect.center(), GROUP_MENU_ICON_RADIUS, color32);

                                                            if ui.selectable_label(false, &g.name).clicked() {
                                                                tab_action = Some(AppAction::AddTabToGroup {
                                                                    group_id: g.id.clone(),
                                                                    member: doc.path.clone(),
                                                                });
                                                                ui.close();
                                                            }
                                                        });
                                                    }
                                                });
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
                && let Some(idx) = self.active_doc_idx {
                    let new_idx = crate::shell_logic::prev_tab_index(idx, doc_count);
                    tab_action = Some(AppAction::SelectDocument(
                        self.open_documents[new_idx].path.clone(),
                    ));
                    ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
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
                && let Some(idx) = self.active_doc_idx {
                    let new_idx = crate::shell_logic::next_tab_index(idx, doc_count);
                    tab_action = Some(AppAction::SelectDocument(
                        self.open_documents[new_idx].path.clone(),
                    ));
                    ui.memory_mut(|m| m.data.insert_temp(egui::Id::new("scroll_tab_req"), true));
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

                if to_visual > 0 && to_visual < tab_rects.len() {
                    let prev_idx = tab_rects[to_visual - 1].0;
                    let next_idx = tab_rects[to_visual].0;

                    if let (Some(prev_doc), Some(next_doc)) = (
                        self.open_documents.get(prev_idx),
                        self.open_documents.get(next_idx),
                    ) {
                        let prev_path = prev_doc.path.to_string_lossy().to_string();
                        let next_path = next_doc.path.to_string_lossy().to_string();

                        for g in self.tab_groups {
                            if g.members.contains(&prev_path) && g.members.contains(&next_path) {
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
    pub show_search: bool,
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
        show_search: bool,
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
            show_search,
        }
    }

    #[allow(deprecated)]
    pub fn show(
        self,
        ui: &mut egui::Ui,
        search_state: &mut crate::state::search::SearchState,
    ) -> Option<AppAction> {
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
                    let badge_str = crate::i18n::get().update.update_available.clone();
                    let badge_color = crate::theme_bridge::from_rgb(0, COLOR_SUCCESS_G, 100);
                    let badge_text = egui::RichText::new(badge_str).color(badge_color).strong();

                    let btn = egui::Button::image_and_text(
                        crate::icon::Icon::Action
                            .image(crate::icon::IconSize::Small)
                            .tint(badge_color),
                        badge_text,
                    )
                    .sense(egui::Sense::click());

                    if ui
                        .add(btn)
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
                            invisible_label("Refresh"),
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
                    let order_tip = match current_order {
                        katana_platform::PaneOrder::EditorFirst => {
                            crate::i18n::get().split_toggle.preview_first.clone()
                        }
                        katana_platform::PaneOrder::PreviewFirst => {
                            crate::i18n::get().split_toggle.editor_first.clone()
                        }
                    };

                    let order_icon =
                        if self.split_direction == katana_platform::SplitDirection::Horizontal {
                            crate::icon::Icon::SwapHorizontal
                        } else {
                            crate::icon::Icon::SwapVertical
                        };

                    if ui
                        .add(egui::Button::image(
                            order_icon
                                .image(crate::icon::IconSize::Medium)
                                .tint(ui.visuals().text_color()),
                        ))
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

                // Add the search button to the far left of the right-aligned layout
                if self.show_search {
                    ui.separator();
                    let doc_search_tooltip =
                        format!("{} (Cmd+F)", crate::i18n::get().search.doc_search_title);

                    let btn_color = if search_state.doc_search_open {
                        ui.visuals().widgets.active.bg_fill
                    } else {
                        egui::Color32::default()
                    };

                    let toggle_resp = ui
                        .add(
                            egui::Button::image(
                                crate::Icon::Search.ui_image(ui, crate::icon::IconSize::Medium),
                            )
                            .fill(btn_color),
                        )
                        .on_hover_text(doc_search_tooltip);

                    if toggle_resp.clicked() {
                        if search_state.doc_search_open {
                            search_state.doc_search_open = false;
                        } else {
                            action = Some(AppAction::OpenDocSearch);
                        }
                    }
                }
            },
        );

        if self.show_search && search_state.doc_search_open {
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(available_width, bar_height),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    // Drawing right-to-left, so we add: Close, Next, Prev, MatchCount, Input
                    if ui
                        .add(egui::Button::image(
                            crate::Icon::Close.ui_image(ui, crate::icon::IconSize::Medium),
                        ))
                        .on_hover_text(crate::i18n::get().search.doc_search_close.clone())
                        .clicked()
                    {
                        search_state.doc_search_open = false;
                    }

                    if ui
                        .add(egui::Button::image(
                            crate::Icon::PanDown.ui_image(ui, crate::icon::IconSize::Medium),
                        ))
                        .on_hover_text(crate::i18n::get().search.doc_search_next.clone())
                        .clicked()
                    {
                        action = Some(AppAction::DocSearchNext);
                    }

                    if ui
                        .add(egui::Button::image(
                            crate::Icon::PanUp.ui_image(ui, crate::icon::IconSize::Medium),
                        ))
                        .on_hover_text(crate::i18n::get().search.doc_search_prev.clone())
                        .clicked()
                    {
                        action = Some(AppAction::DocSearchPrev);
                    }

                    let match_count = search_state.doc_search_matches.len();
                    if match_count > 0 {
                        ui.label(crate::i18n::tf(
                            &crate::i18n::get().search.doc_search_count,
                            &[
                                (
                                    "index",
                                    &format!("{}", search_state.doc_search_active_index + 1),
                                ),
                                ("total", &format!("{}", match_count)),
                            ],
                        ));
                    } else if !search_state.doc_search_query.is_empty() {
                        ui.label(crate::i18n::tf(
                            &crate::i18n::get().search.doc_search_count,
                            &[("index", "0"), ("total", "0")],
                        ));
                    }

                    const DOC_SEARCH_INPUT_WIDTH: f32 = 200.0;
                    let is_newly_opened = ui.memory(|m| {
                        m.data
                            .get_temp::<bool>(egui::Id::new("search_newly_opened"))
                            .unwrap_or(false)
                    });

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut search_state.doc_search_query)
                            .desired_width(DOC_SEARCH_INPUT_WIDTH)
                            .id_source("doc_search_input_stable_id"),
                    );

                    // Add an inline clear button 'x' overlaying the right side of the TextEdit
                    if !search_state.doc_search_query.is_empty() {
                        const DOC_SEARCH_CLEAR_BTN_WIDTH: f32 = 20.0;
                        const DOC_SEARCH_CLEAR_BTN_FONT_SIZE: f32 = 14.0;
                        const DOC_SEARCH_CLEAR_BTN_ALPHA: f32 = 0.5;

                        let rect = response.rect;
                        let btn_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.max.x - DOC_SEARCH_CLEAR_BTN_WIDTH, rect.min.y),
                            egui::vec2(DOC_SEARCH_CLEAR_BTN_WIDTH, rect.height()),
                        );
                        ui.allocate_ui_at_rect(btn_rect, |ui| {
                            let btn = egui::Button::new(
                                egui::RichText::new("×")
                                    .size(DOC_SEARCH_CLEAR_BTN_FONT_SIZE)
                                    .color(
                                        ui.visuals()
                                            .text_color()
                                            .gamma_multiply(DOC_SEARCH_CLEAR_BTN_ALPHA),
                                    ),
                            )
                            .frame(false);

                            if ui.centered_and_justified(|ui| ui.add(btn)).inner.clicked() {
                                search_state.doc_search_query.clear();
                                action = Some(AppAction::DocSearchQueryChanged);
                                response.request_focus();
                            }
                        });
                    }

                    if is_newly_opened {
                        response.request_focus();
                        ui.memory_mut(|m| {
                            m.data
                                .insert_temp(egui::Id::new("search_newly_opened"), false)
                        });
                    }

                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let shift_pressed = ui.input(|i| i.modifiers.shift);
                    let up_pressed =
                        response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                    let down_pressed =
                        response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::ArrowDown));

                    if (enter_pressed && !shift_pressed) || down_pressed {
                        action = Some(AppAction::DocSearchNext);
                    } else if (enter_pressed && shift_pressed) || up_pressed {
                        action = Some(AppAction::DocSearchPrev);
                    }

                    if response.changed() {
                        action = Some(AppAction::DocSearchQueryChanged);
                    }

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        search_state.doc_search_open = false;
                    }
                },
            );
        }

        if mode != prev {
            action = Some(AppAction::SetViewMode(mode));
        }

        action
    }
}
