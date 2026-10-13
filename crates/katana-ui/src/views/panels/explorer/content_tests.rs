#[cfg(test)]
mod tests {
    use crate::app_state::{AppAction, SearchState, WorkspaceState};
    use crate::views::panels::explorer::content::ExplorerContent;
    use egui_kittest::Harness;
    use egui_kittest::kittest::{NodeT, Queryable};
    use katana_core::workspace::{TreeEntry, Workspace};
    use std::collections::HashSet;
    use std::path::Path;
    use std::path::PathBuf;

    struct ExplorerHarnessState {
        workspace: WorkspaceState,
        search: SearchState,
        action: AppAction,
    }

    fn large_explorer_state() -> ExplorerHarnessState {
        let root = PathBuf::from("/workspace");
        let docs = root.join("docs");
        let children = (0..10_000)
            .map(|index| TreeEntry::File {
                path: docs.join(format!("file-{index}.pdf")),
            })
            .collect();
        let mut workspace = WorkspaceState::new();
        workspace.data = Some(Workspace::new(
            &root,
            vec![TreeEntry::Directory {
                path: docs.clone(),
                children,
            }],
        ));
        workspace.expanded_directories = HashSet::from([docs]);
        ExplorerHarnessState {
            workspace,
            search: SearchState::new(),
            action: AppAction::None,
        }
    }

    #[test]
    fn workspace_root_context_actions_target_workspace_root() {
        let workspace_root = Path::new("/workspace");

        assert_eq!(
            ExplorerContent::format_workspace_markdown_action(workspace_root),
            AppAction::FormatWorkspaceMarkdown(workspace_root.to_path_buf())
        );
        assert_eq!(
            ExplorerContent::new_workspace_root_file_action(workspace_root),
            AppAction::RequestNewFile(workspace_root.to_path_buf())
        );
        assert_eq!(
            ExplorerContent::new_workspace_root_directory_action(workspace_root),
            AppAction::RequestNewDirectory(workspace_root.to_path_buf())
        );
    }

    #[test]
    fn large_document_tree_scrolls_with_bounded_interactive_rows() {
        let mut harness = Harness::builder()
            .with_size(eframe::egui::vec2(320.0, 240.0))
            .build_ui_state(
                |ui, state: &mut ExplorerHarnessState| {
                    ExplorerContent::new(
                        &mut state.workspace,
                        &mut state.search,
                        &[],
                        None,
                        &[],
                        &mut state.action,
                        true,
                    )
                    .show(ui);
                },
                large_explorer_state(),
            );

        let initial_files = harness
            .get_all_by_role(eframe::egui::accesskit::Role::Button)
            .filter_map(|node| node.accesskit_node().label())
            .filter(|label| label.starts_with("file "))
            .collect::<Vec<_>>();
        assert!(!initial_files.is_empty());
        assert!(initial_files.len() <= 16);

        let first_file = harness.get_by_label("file file-0.pdf");
        for _ in 0..8 {
            first_file.scroll_down();
        }
        harness.run();

        let scrolled_files = harness
            .get_all_by_role(eframe::egui::accesskit::Role::Button)
            .filter_map(|node| node.accesskit_node().label())
            .filter(|label| label.starts_with("file "))
            .collect::<Vec<_>>();
        assert!(!scrolled_files.is_empty());
        assert!(scrolled_files.len() <= 16);
        assert!(
            !scrolled_files
                .iter()
                .any(|label| label == "file file-0.pdf")
        );

        let target_label = scrolled_files[0].clone();
        harness.get_by_label(&target_label).click();
        harness.run();
        let expected = PathBuf::from("/workspace/docs").join(&target_label[5..]);
        assert!(matches!(
            &harness.state().action,
            AppAction::SelectDocument(path) if path == &expected
        ));
    }
}
