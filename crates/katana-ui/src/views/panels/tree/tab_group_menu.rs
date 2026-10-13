use crate::shell_ui::TreeRenderContext;
use eframe::egui;

pub(crate) struct TabGroupMenu;

impl TabGroupMenu {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        ctx: &mut TreeRenderContext,
        is_dir: bool,
        children: Option<&[katana_core::workspace::TreeEntry]>,
    ) {
        if ui
            .button(&crate::i18n::I18nOps::get().tab.create_new_group)
            .clicked()
        {
            Self::handle_create_group_click(ctx, path, is_dir, children);
            ui.close();
        }

        if let Some(groups) = ctx.tab_groups
            && !groups.is_empty()
        {
            ui.separator();
            for g in groups {
                if g.is_demo() {
                    continue;
                }
                if ui.button(&g.name).clicked() {
                    Self::handle_add_to_group_click(ctx, path, g.id.clone(), is_dir, children);
                    ui.close();
                }
            }
        }
    }

    fn handle_create_group_click(
        ctx: &mut TreeRenderContext,
        path: &std::path::Path,
        is_dir: bool,
        children: Option<&[katana_core::workspace::TreeEntry]>,
    ) {
        if is_dir {
            if let Some(children) = children {
                let mut paths = Vec::new();
                for c in children {
                    c.collect_all_markdown_file_paths(&mut paths);
                }
                if !paths.is_empty() {
                    let name = path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    *ctx.action = crate::app_state::AppAction::CreateTabGroupMany {
                        name,
                        color_hex: "#4A90D9".to_string(),
                        members: paths,
                    };
                }
            }
        } else {
            *ctx.action = crate::app_state::AppAction::CreateTabGroup {
                name: "".to_string(),
                color_hex: "#4A90D9".to_string(),
                initial_member: path.to_path_buf(),
            };
        }
    }

    fn handle_add_to_group_click(
        ctx: &mut TreeRenderContext,
        path: &std::path::Path,
        group_id: String,
        is_dir: bool,
        children: Option<&[katana_core::workspace::TreeEntry]>,
    ) {
        if is_dir {
            if let Some(children) = children {
                let mut paths = Vec::new();
                for c in children {
                    c.collect_all_markdown_file_paths(&mut paths);
                }
                if !paths.is_empty() {
                    *ctx.action = crate::app_state::AppAction::AddTabsToGroup {
                        group_id,
                        members: paths,
                    };
                }
            }
        } else {
            *ctx.action = crate::app_state::AppAction::AddTabToGroup {
                group_id,
                member: path.to_path_buf(),
            };
        }
    }
}
