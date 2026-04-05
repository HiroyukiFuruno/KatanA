use crate::app_state::AppAction;
use crate::views::top_bar::types::TopBarOps;
use eframe::egui;

pub(crate) struct TabDragHandler<'a> {
    pub src_idx: usize,
    pub ghost_center_x: f32,
    pub tab_rects: &'a [(usize, egui::Rect)],
    pub open_documents: &'a [katana_core::document::Document],
    pub tab_groups: &'a [crate::state::document::TabGroup],
}

impl<'a> TabDragHandler<'a> {
    pub fn resolve(self) -> Option<AppAction> {
        let drop_points = TopBarOps::compute_drop_points(self.tab_rects);
        let to_visual = TopBarOps::find_best_drop_index(&drop_points, self.ghost_center_x)?;
        let to_physical = self.visual_to_physical(to_visual);
        let new_group_id = self.resolve_group_at(to_visual);
        self.build_action(to_physical, new_group_id)
    }

    fn visual_to_physical(&self, to_visual: usize) -> usize {
        if to_visual < self.tab_rects.len() {
            self.tab_rects[to_visual].0
        } else {
            self.open_documents.len()
        }
    }

    fn resolve_group_at(&self, to_visual: usize) -> Option<Option<String>> {
        if to_visual == 0 || to_visual >= self.tab_rects.len() {
            return Some(None);
        }
        let prev_idx = self.tab_rects[to_visual - 1].0;
        let next_idx = self.tab_rects[to_visual].0;
        let prev_doc = self.open_documents.get(prev_idx)?;
        let next_doc = self.open_documents.get(next_idx)?;
        let prev_path = prev_doc.path.display().to_string();
        let next_path = next_doc.path.display().to_string();
        for g in self.tab_groups {
            if g.members.contains(&prev_path) && g.members.contains(&next_path) {
                return Some(Some(g.id.clone()));
            }
        }
        Some(None)
    }

    fn build_action(
        &self,
        to_physical: usize,
        new_group_id: Option<Option<String>>,
    ) -> Option<AppAction> {
        let src = self.src_idx;
        if src != to_physical && src + 1 != to_physical {
            return Some(AppAction::ReorderDocument { from: src, to: to_physical, new_group_id });
        }
        self.maybe_group_change_action(new_group_id)
    }

    fn maybe_group_change_action(
        &self,
        new_group_id: Option<Option<String>>,
    ) -> Option<AppAction> {
        let src = self.src_idx;
        let path_str = self.open_documents[src].path.display().to_string();
        match &new_group_id {
            Some(Some(g_id)) => {
                let in_group = self
                    .tab_groups
                    .iter()
                    .find(|g| g.id == *g_id)
                    .is_some_and(|g| g.members.contains(&path_str));
                if !in_group {
                    Some(AppAction::ReorderDocument { from: src, to: src, new_group_id })
                } else {
                    None
                }
            }
            Some(None) => {
                let in_any = self.tab_groups.iter().any(|g| g.members.contains(&path_str));
                if in_any {
                    Some(AppAction::ReorderDocument { from: src, to: src, new_group_id })
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
