use crate::app::doc_search::DocSearchRefresh;
use crate::app_state::*;
use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_toggle_slideshow(&mut self, ctx: &egui::Context) {
        self.state.layout.show_slideshow = !self.state.layout.show_slideshow;
        if self.state.layout.show_slideshow {
            self.state.layout.slideshow_page = 0;
            let is_fs = ctx
                .input(|i: &egui::InputState| i.viewport().fullscreen)
                .unwrap_or(false);
            self.state.layout.was_os_fullscreen_before_slideshow = is_fs;
            if !is_fs {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
        } else if !self.state.layout.was_os_fullscreen_before_slideshow {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }
    }

    pub(super) fn handle_action_doc_search_changed(&mut self) {
        let Some(doc) = self.state.document.active_document() else {
            return;
        };
        let text = doc.buffer.clone();
        self.refresh_doc_search_matches(&text);
        if let Some(r) = self.state.search.doc_search_matches.first() {
            let line = crate::views::panels::editor::types::EditorLogicOps::char_index_to_line(
                &text, r.start,
            );
            self.state.scroll.scroll_to_line = Some(line);
        }
    }

    pub(super) fn handle_action_doc_search_next(&mut self, ctx: &egui::Context) {
        let Some(doc) = self.state.document.active_document() else {
            return;
        };
        if let Some(result) = crate::app::doc_search::DocSearchOps::navigate_next(
            &self.state.search.doc_search_matches,
            self.state.search.doc_search_active_index,
            &doc.buffer,
        ) {
            self.state.search.doc_search_active_index = result.new_active_index;
            self.state.scroll.last_scroll_to_line = None;
            self.state.scroll.scroll_to_line = result.scroll_to_line;
            /* WHY: PreviewOnly mode doesn't render the editor, so scroll_to_line
             * is never consumed. Trigger the preview's own scroll mechanism instead. */
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("katana_preview_search_scroll_pending"), true);
            });
        }
    }

    pub(super) fn handle_action_doc_search_prev(&mut self, ctx: &egui::Context) {
        let Some(doc) = self.state.document.active_document() else {
            return;
        };
        if let Some(result) = crate::app::doc_search::DocSearchOps::navigate_prev(
            &self.state.search.doc_search_matches,
            self.state.search.doc_search_active_index,
            &doc.buffer,
        ) {
            self.state.search.doc_search_active_index = result.new_active_index;
            self.state.scroll.last_scroll_to_line = None;
            self.state.scroll.scroll_to_line = result.scroll_to_line;
            /* WHY: PreviewOnly mode doesn't render the editor, so scroll_to_line
             * is never consumed. Trigger the preview's own scroll mechanism instead. */
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("katana_preview_search_scroll_pending"), true);
            });
        }
    }

    pub(crate) fn handle_action_refresh_diagnostics(&mut self) {
        let Some(doc) = self.state.active_document() else {
            return;
        };
        let path = doc.path.clone();
        let content = doc.buffer.clone();

        /* WHY: Virtual documents (e.g. "Katana://LinterDocs/MD*.md", "Katana://Demo/...") are not real
         * filesystem files; running the linter on them would produce spurious diagnostics
         * shown to the user without a backing file. Skip any path starting with "Katana://".
         * Exception: "lint-fix.md" is explicitly designed to demonstrate the linter. */
        use crate::state::document::VirtualPathExt as _;
        if path.is_virtual_path() {
            let path_str = path.to_string_lossy();
            if !path_str.ends_with("lint-fix.md") && !path_str.ends_with("lint-fix.ja.md") {
                return;
            }
        }

        let is_markdown = path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
            .unwrap_or(false);
        if !is_markdown {
            return;
        }

        let linter_settings = &self.state.config.settings.settings().linter;
        let enabled = linter_settings.enabled;
        let mut severity_map = std::collections::HashMap::new();
        for (rule_id, severity) in &linter_settings.rule_severity {
            let mapped_severity = match severity {
                katana_platform::settings::types::RuleSeverity::Ignore => None,
                katana_platform::settings::types::RuleSeverity::Warning => {
                    Some(katana_linter::rules::markdown::DiagnosticSeverity::Warning)
                }
                katana_platform::settings::types::RuleSeverity::Error => {
                    Some(katana_linter::rules::markdown::DiagnosticSeverity::Error)
                }
            };
            severity_map.insert(rule_id.clone(), mapped_severity);
        }

        let diagnostics = katana_linter::rules::markdown::MarkdownLinterOps::evaluate_all(
            &path,
            &content,
            enabled,
            &severity_map,
        );
        self.state.diagnostics.update_diagnostics(path, diagnostics);
    }

    pub(super) fn handle_action_clear_all_caches(&mut self, ctx: &egui::Context) {
        use egui::load::BytesLoader;
        katana_platform::cache::DefaultCacheService::clear_all_directories();
        crate::http_cache_loader::PersistentHttpLoader::default().forget_all();
        ctx.forget_all_images();
        crate::icon::IconRegistry::install(ctx);
        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::get()
                .settings
                .behavior
                .clear_http_cache
                .clone(),
            StatusType::Success,
        ));
    }

    pub(super) fn handle_action_reorder_activity_rail(&mut self, from: usize, to: usize) {
        let mut order = self
            .state
            .config
            .settings
            .settings()
            .layout
            .activity_rail_order
            .clone();
        let len = order.len();
        if from < len && to <= len && from != to {
            let item = order.remove(from);
            let actual_to = if to > from { to - 1 } else { to };
            order.insert(actual_to, item);
            self.state
                .config
                .settings
                .settings_mut()
                .layout
                .activity_rail_order = order;
            if !self.state.config.try_save_settings() {
                tracing::warn!("Failed to save activity rail reorder");
            }
        }
    }

    pub(super) fn handle_action_reveal_in_os(&self, path: std::path::PathBuf) {
        #[cfg(target_os = "macos")]
        {
            let _ = katana_core::system::ProcessService::create_command("open")
                .arg("-R")
                .arg(&path)
                .spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = katana_core::system::ProcessService::create_command("explorer")
                .arg("/select,")
                .arg(&path)
                .spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let dir = if path.is_file() {
                path.parent().unwrap_or(&path)
            } else {
                &path
            };
            let _ = katana_core::system::ProcessService::create_command("xdg-open")
                .arg(dir)
                .spawn();
        }
    }
}
