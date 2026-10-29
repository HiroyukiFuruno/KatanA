use eframe::egui;

#[derive(Debug, Clone, Copy)]
pub(super) struct GroupHeaderPopupState {
    pub(super) popup_id: egui::Id,
    pub(super) name_draft_id: egui::Id,
    name_focus_id: egui::Id,
    opened_for_inline_rename_id: egui::Id,
}

impl GroupHeaderPopupState {
    pub(super) fn new(group_id: &str) -> Self {
        Self::from_popup_id(egui::Id::new("group_popup").with(group_id))
    }

    pub(super) fn from_popup_id(popup_id: egui::Id) -> Self {
        Self {
            popup_id,
            name_draft_id: popup_id.with("group_name_draft"),
            name_focus_id: popup_id.with("group_name_focus_requested"),
            opened_for_inline_rename_id: popup_id.with("opened_for_inline_rename"),
        }
    }

    pub(super) fn open_inline_once(self, ctx: &egui::Context) {
        if self.inline_was_opened(ctx) {
            return;
        }
        /* WHY: 外側クリック後、取消処理中にポップアップを再表示しない。 */
        egui::Popup::open_id(ctx, self.popup_id);
        ctx.data_mut(|data| data.insert_temp(self.opened_for_inline_rename_id, true));
    }

    pub(super) fn inline_was_opened(self, ctx: &egui::Context) -> bool {
        ctx.data_mut(|data| {
            data.get_temp::<bool>(self.opened_for_inline_rename_id)
                .unwrap_or(false)
        })
    }

    pub(super) fn request_initial_focus(self, ctx: &egui::Context, inline_rename: bool) -> bool {
        if !inline_rename {
            return false;
        }
        let requested =
            ctx.data_mut(|data| data.get_temp::<bool>(self.name_focus_id).unwrap_or(false));
        if !requested {
            /* WHY: 毎フレームfocusを奪い返すと確定/取消ボタンを操作できない。 */
            ctx.data_mut(|data| data.insert_temp(self.name_focus_id, true));
        }
        !requested
    }

    pub(super) fn clear_editor_state(self, ctx: &egui::Context) {
        ctx.data_mut(|data| {
            data.remove_temp::<String>(self.name_draft_id);
            data.remove_temp::<bool>(self.name_focus_id);
        });
    }

    pub(super) fn clear_open_marker(self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.remove_temp::<bool>(self.opened_for_inline_rename_id));
    }
}
