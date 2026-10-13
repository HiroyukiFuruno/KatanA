use crate::shell::KatanaApp;
use eframe::egui;

/// Which popup panel button is being hovered (for delay tracking).
#[derive(Clone, Copy, PartialEq, Default)]
pub(super) enum PendingPanel {
    #[default]
    None,
    Export,
    Story,
    Tools,
}

/// Persistent hover-delay state stored in egui temp data.
#[derive(Clone, Copy, Default)]
pub(super) struct HoverDelay {
    pub(super) pending: PendingPanel,
    pub(super) start_time: f64,
}

pub struct PreviewSidePanels<'a> {
    pub app: &'a mut KatanaApp,
    pub(super) export_btn_rect: Option<egui::Rect>,
    pub(super) story_btn_rect: Option<egui::Rect>,
    pub(super) tools_btn_rect: Option<egui::Rect>,
    pub(super) toc_btn_rect: Option<egui::Rect>,
    /// Screen-space rect of the sidebar, used to position overlay popups.
    pub(super) sidebar_rect: Option<egui::Rect>,
}

impl<'a> PreviewSidePanels<'a> {
    pub fn new(app: &'a mut KatanaApp) -> Self {
        Self {
            app,
            export_btn_rect: None,
            story_btn_rect: None,
            tools_btn_rect: None,
            toc_btn_rect: None,
            sidebar_rect: None,
        }
    }
}
