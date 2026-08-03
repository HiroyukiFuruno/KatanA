#![allow(unused_imports)]
#![allow(dead_code)]
mod render;
mod source;

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

use render::{
    FullPreviewRenderOptions, full_render_preview_pane, preview_hash_for_path, update_preview_hash,
    update_preview_pane,
};
use source::{is_html_preview_path, markdown_preview_source_for_path};

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
    fn full_refresh_document_source(
        &mut self,
        path: &std::path::Path,
        source: crate::preview_pane::DocumentSurfaceSource,
        force: bool,
    );
    fn full_refresh_document_failure(
        &mut self,
        path: &std::path::Path,
        failure: crate::preview_pane::DocumentFailure,
    );
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
        if katana_core::workspace::TreeEntry::path_is_document(path) {
            match crate::preview_pane::DocumentSurfaceSource::local(path) {
                Ok(source) => self.full_refresh_document_source(path, source, force),
                Err(error) => {
                    self.full_refresh_document_failure(path, error.clone());
                    self.state.layout.status_message = Some((error.to_string(), StatusType::Error));
                }
            }
            return;
        }
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

        if is_html
            && let Some(origin) = self
                .state
                .url_tab
                .source_for_document(path)
                .map(|source| source.origin.clone())
        {
            let browser_source =
                match katana_document_viewer::browser_session::HtmlBrowserSource::new(
                    actual_source.clone(),
                    origin,
                ) {
                    Ok(source) => source,
                    Err(error) => {
                        self.state.layout.status_message =
                            Some((error.to_string(), StatusType::Error));
                        return;
                    }
                };
            let pane = Self::get_preview_pane(&mut self.tab_previews, path_buf.clone());
            pane.full_render_html_source(browser_source, force);
            update_preview_hash(&mut self.tab_previews, &path_buf, h);
            return;
        }

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

    fn full_refresh_document_source(
        &mut self,
        path: &std::path::Path,
        source: crate::preview_pane::DocumentSurfaceSource,
        force: bool,
    ) {
        let path = path.to_path_buf();
        let pane = Self::get_preview_pane(&mut self.tab_previews, path.clone());
        pane.full_render_document_source(source, force);
        update_preview_hash(&mut self.tab_previews, &path, 0);
    }

    fn full_refresh_document_failure(
        &mut self,
        path: &std::path::Path,
        failure: crate::preview_pane::DocumentFailure,
    ) {
        let path = path.to_path_buf();
        let pane = Self::get_preview_pane(&mut self.tab_previews, path.clone());
        pane.full_render_document_failure(failure);
        update_preview_hash(&mut self.tab_previews, &path, 0);
    }
}
