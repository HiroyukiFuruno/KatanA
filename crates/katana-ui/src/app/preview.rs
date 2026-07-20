#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::preview_pane::PreviewPane;
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) trait PreviewOps {
    fn get_preview_pane(
        previews: &mut Vec<TabPreviewCache>,
        path: std::path::PathBuf,
    ) -> &mut PreviewPane;
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str);
    fn full_refresh_preview(
        &mut self,
        path: &std::path::Path,
        source: &str,
        force: bool,
        concurrency: usize,
    );
    fn full_refresh_html_source(
        &mut self,
        path: &std::path::Path,
        source: katana_document_viewer::browser_session::HtmlBrowserSource,
    );
}

fn is_drawio_preview_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("drawio") || e.eq_ignore_ascii_case("drowio"))
        .unwrap_or(false)
}

fn is_html_preview_path(path: &std::path::Path) -> bool {
    katana_core::workspace::TreeEntry::path_is_html(path)
}

fn markdown_preview_source_for_path(path: &std::path::Path, source: &str) -> String {
    if is_drawio_preview_path(path) {
        format!("```drawio\n{}\n```", source)
    } else {
        source.to_string()
    }
}

impl PreviewOps for KatanaApp {
    fn get_preview_pane(
        previews: &mut Vec<TabPreviewCache>,
        path: std::path::PathBuf,
    ) -> &mut PreviewPane {
        if let Some(idx) = previews.iter().position(|t| t.path == path) {
            &mut previews[idx].pane
        } else {
            previews.push(TabPreviewCache {
                path,
                pane: PreviewPane::default(),
                hash: 0,
            });
            &mut previews.last_mut().expect("just pushed").pane
        }
    }
    fn refresh_preview(&mut self, path: &std::path::Path, source: &str) {
        let is_html = is_html_preview_path(path);
        let actual_source = markdown_preview_source_for_path(path, source);
        let h = ShellLogicOps::hash_str(&actual_source);
        let path_buf = path.to_path_buf();

        if preview_hash_for_path(&self.tab_previews, &path_buf).is_some_and(|hash| hash == h) {
            return;
        }

        let pane = Self::get_preview_pane(&mut self.tab_previews, path_buf.clone());
        update_preview_pane(pane, path, &actual_source, is_html);
        update_preview_hash(&mut self.tab_previews, &path_buf, h);
    }

    fn full_refresh_preview(
        &mut self,
        path: &std::path::Path,
        source: &str,
        force: bool,
        concurrency: usize,
    ) {
        let is_html = is_html_preview_path(path);
        let actual_source = markdown_preview_source_for_path(path, source);
        let h = ShellLogicOps::hash_str(&actual_source);
        let path_buf = path.to_path_buf();
        let current_hash = preview_hash_for_path(&self.tab_previews, &path_buf).unwrap_or(0);

        if !force && current_hash != 0 && current_hash == h {
            return;
        }

        tracing::debug!(
            "[DEBUG-HASH] MISMATCH or FORCE. Running full_render for path: {:?}. force={}, current_hash={}, new_hash={}",
            path_buf,
            force,
            current_hash,
            h
        );

        let pane = Self::get_preview_pane(&mut self.tab_previews, path_buf.clone());
        full_render_preview_pane(
            pane,
            path,
            &actual_source,
            FullPreviewRenderOptions {
                is_html,
                force,
                concurrency,
                cache: self.state.config.cache.clone(),
            },
        );
        update_preview_hash(&mut self.tab_previews, &path_buf, h);
    }

    fn full_refresh_html_source(
        &mut self,
        path: &std::path::Path,
        source: katana_document_viewer::browser_session::HtmlBrowserSource,
    ) {
        let path = path.to_path_buf();
        let pane = Self::get_preview_pane(&mut self.tab_previews, path.clone());
        pane.full_render_html_source(source, true);
        update_preview_hash(&mut self.tab_previews, &path, 0);
    }
}

fn preview_hash_for_path(previews: &[TabPreviewCache], path: &std::path::Path) -> Option<u64> {
    previews
        .iter()
        .find(|tab| tab.path == path)
        .map(|tab| tab.hash)
}

fn update_preview_hash(previews: &mut [TabPreviewCache], path: &std::path::Path, hash: u64) {
    if let Some(tab) = previews.iter_mut().find(|tab| tab.path == path) {
        tab.hash = hash;
    }
}

fn update_preview_pane(
    pane: &mut PreviewPane,
    path: &std::path::Path,
    source: &str,
    is_html: bool,
) {
    if is_html {
        pane.update_html_document_sections(source, path);
    } else {
        pane.update_markdown_sections(source, path);
    }
}

struct FullPreviewRenderOptions {
    is_html: bool,
    force: bool,
    concurrency: usize,
    cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
}

fn full_render_preview_pane(
    pane: &mut PreviewPane,
    path: &std::path::Path,
    source: &str,
    options: FullPreviewRenderOptions,
) {
    if options.is_html {
        pane.full_render_html_document(source, path, options.force);
    } else {
        pane.full_render(
            source,
            path,
            options.cache,
            options.force,
            options.concurrency,
        );
    }
}
