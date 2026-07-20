use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

pub(crate) struct HtmlPreviewObserver {
    target: PathBuf,
    _watcher: RecommendedWatcher,
    events: Receiver<notify::Result<notify::Event>>,
}

impl HtmlPreviewObserver {
    pub(crate) fn for_file(path: &Path) -> notify::Result<Self> {
        let target = path.to_path_buf();
        let watch_root = target.parent().unwrap_or_else(|| Path::new("."));
        let (tx, events) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(watcher_event_callback(tx), Config::default())?;
        watcher.watch(watch_root, RecursiveMode::NonRecursive)?;
        Ok(Self {
            target,
            _watcher: watcher,
            events,
        })
    }

    pub(crate) fn target(&self) -> &Path {
        &self.target
    }

    pub(crate) fn has_target_change(&self) -> bool {
        let mut changed = false;
        loop {
            match self.events.try_recv() {
                Ok(Ok(event)) => {
                    changed |= event.paths.iter().any(|path| path == &self.target);
                }
                Ok(Err(error)) => {
                    tracing::warn!(target = %self.target.display(), %error, "HTML preview watcher error");
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => return changed,
            }
        }
    }
}

impl crate::shell::KatanaApp {
    pub(crate) fn sync_html_preview_observer(&mut self) {
        let Some(path) = self.state.active_path() else {
            self.html_preview_observer = None;
            return;
        };
        self.observe_html_preview_path(&path);
    }

    pub(crate) fn observe_html_preview_path(&mut self, path: &Path) {
        if !katana_core::workspace::TreeEntry::path_is_html(path) {
            self.html_preview_observer = None;
            return;
        }
        if self
            .html_preview_observer
            .as_ref()
            .is_some_and(|observer| observer.target() == path)
        {
            return;
        }
        self.html_preview_observer =
            html_preview_observer_from_result(path, HtmlPreviewObserver::for_file(path));
    }

    pub(crate) fn poll_html_preview_observer(&mut self) {
        if !matches!(self.pending_action, crate::app_state::AppAction::None) {
            return;
        }
        let changed = self
            .html_preview_observer
            .as_ref()
            .is_some_and(HtmlPreviewObserver::has_target_change);
        if changed {
            self.pending_action = crate::app_state::AppAction::RefreshDocument { is_manual: false };
        }
    }
}

fn html_preview_observer_from_result(
    path: &Path,
    result: notify::Result<HtmlPreviewObserver>,
) -> Option<HtmlPreviewObserver> {
    match result {
        Ok(observer) => Some(observer),
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "HTML preview watcher setup failed");
            None
        }
    }
}

fn watcher_event_callback(
    sender: Sender<notify::Result<notify::Event>>,
) -> impl FnMut(notify::Result<notify::Event>) + Send + 'static {
    move |event| {
        let _ = sender.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{AppAction, AppState};
    use crate::shell::KatanaApp;
    use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
    use notify::{Event, EventKind};
    use std::sync::mpsc::{Receiver, channel};

    #[test]
    fn observer_reports_only_target_events_and_tolerates_watcher_errors() {
        let target = PathBuf::from("workspace/index.html");
        let other = PathBuf::from("workspace/other.html");
        let (sender, receiver) = channel();
        sender.send(Ok(event_for(other))).unwrap();
        sender
            .send(Err(notify::Error::generic("test watcher failure")))
            .unwrap();
        sender.send(Ok(event_for(target.clone()))).unwrap();
        drop(sender);
        let observer = observer_with_events(target.clone(), receiver);

        assert_eq!(observer.target(), target);
        assert!(observer.has_target_change());
    }

    #[test]
    fn watcher_callback_forwards_events_to_the_observer_queue() {
        let target = PathBuf::from("workspace/index.html");
        let (sender, receiver) = channel();
        let mut callback = watcher_event_callback(sender);
        callback(Ok(event_for(target.clone())));

        let observer = observer_with_events(target, receiver);
        assert!(observer.has_target_change());
    }

    #[test]
    fn observer_registers_a_real_file_watch() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("index.html");
        std::fs::write(&target, "initial").unwrap();
        let observer = HtmlPreviewObserver::for_file(&target).unwrap();

        assert_eq!(observer.target(), target);
    }

    #[test]
    fn observer_result_and_app_poll_preserve_auto_refresh_contract() {
        let target = PathBuf::from("workspace/index.html");
        let (_, receiver) = channel();
        let observer = observer_with_events(target.clone(), receiver);

        assert!(html_preview_observer_from_result(&target, Ok(observer)).is_some());
        assert!(
            html_preview_observer_from_result(
                &target,
                Err(notify::Error::generic("test watcher failure")),
            )
            .is_none()
        );

        let (sender, receiver) = channel();
        sender.send(Ok(event_for(target))).unwrap();
        drop(sender);
        let mut app = test_app();
        app.html_preview_observer = Some(observer_with_events(
            PathBuf::from("workspace/index.html"),
            receiver,
        ));
        app.poll_html_preview_observer();

        assert!(matches!(
            app.pending_action,
            AppAction::RefreshDocument { is_manual: false }
        ));
    }

    #[test]
    fn observer_keeps_file_events_queued_while_another_action_is_pending() {
        let target = PathBuf::from("workspace/index.html");
        let (sender, receiver) = channel();
        sender.send(Ok(event_for(target.clone()))).unwrap();
        drop(sender);
        let mut app = test_app();
        app.html_preview_observer = Some(observer_with_events(target, receiver));
        app.pending_action = AppAction::SaveDocument;

        app.poll_html_preview_observer();
        assert!(matches!(app.pending_action, AppAction::SaveDocument));

        app.pending_action = AppAction::None;
        app.poll_html_preview_observer();
        assert!(matches!(
            app.pending_action,
            AppAction::RefreshDocument { is_manual: false }
        ));
    }

    #[test]
    fn app_observes_html_files_and_clears_non_html_targets() {
        let root = tempfile::tempdir().unwrap();
        let html = root.path().join("index.html");
        std::fs::write(&html, "<p>preview</p>").unwrap();
        let mut app = test_app();

        app.observe_html_preview_path(&html);
        assert_eq!(
            app.html_preview_observer
                .as_ref()
                .map(HtmlPreviewObserver::target),
            Some(html.as_path())
        );
        app.observe_html_preview_path(&html);
        app.observe_html_preview_path(&root.path().join("notes.md"));

        assert!(app.html_preview_observer.is_none());
    }

    #[test]
    fn observer_follows_only_the_active_document() {
        let root = tempfile::tempdir().unwrap();
        let html = root.path().join("index.html");
        let markdown = root.path().join("notes.md");
        std::fs::write(&html, "<p>preview</p>").unwrap();
        std::fs::write(&markdown, "# Notes").unwrap();
        let mut app = test_app();
        app.state
            .document
            .open_documents
            .push(katana_core::document::Document::new(
                &html,
                "<p>preview</p>",
            ));
        app.state
            .document
            .open_documents
            .push(katana_core::document::Document::new(&markdown, "# Notes"));

        app.state.document.active_doc_idx = Some(0);
        app.sync_html_preview_observer();
        assert_eq!(
            app.html_preview_observer
                .as_ref()
                .map(HtmlPreviewObserver::target),
            Some(html.as_path())
        );

        app.state.document.active_doc_idx = Some(1);
        app.sync_html_preview_observer();
        assert!(app.html_preview_observer.is_none());
    }

    fn event_for(path: PathBuf) -> Event {
        let mut event = Event::new(EventKind::Any);
        event.paths.push(path);
        event
    }

    fn observer_with_events(
        target: PathBuf,
        events: Receiver<notify::Result<Event>>,
    ) -> HtmlPreviewObserver {
        let watcher = RecommendedWatcher::new(|_| {}, Config::default()).unwrap();
        HtmlPreviewObserver {
            target,
            _watcher: watcher,
            events,
        }
    }

    fn test_app() -> KatanaApp {
        let mut state = AppState::new(
            AiProviderRegistry::new(),
            PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
            katana_platform::workspace::InMemoryWorkspaceRepository::default(),
        ));
        KatanaApp::new(state)
    }
}
