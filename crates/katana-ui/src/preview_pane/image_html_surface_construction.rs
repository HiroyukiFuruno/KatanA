use super::{HtmlBrowserSource, HtmlBrowserSurface};
use std::collections::VecDeque;

impl HtmlBrowserSurface {
    pub(crate) fn start(source: HtmlBrowserSource) -> Self {
        let initial_origin = source.origin.as_str().to_owned();
        let document_origin = Some(initial_origin.clone());
        Self {
            adapter: None,
            katana_adapter_instance_id: None,
            pending_renderer_session: false,
            katana_renderer_session_epoch: None,
            pending_source: Some(source),
            frame: None,
            document_origin,
            texture: None,
            viewport: None,
            last_pointer_position: None,
            primary_pointer_pressed: false,
            focused: false,
            pointer_over: false,
            last_display_rect: None,
            error: None,
            frame_update_deadline: None,
            pending_navigation_urls: VecDeque::new(),
            navigation_history: VecDeque::from([initial_origin]),
        }
    }

    pub(crate) fn failed(error: String) -> Self {
        Self {
            adapter: None,
            katana_adapter_instance_id: None,
            pending_renderer_session: false,
            katana_renderer_session_epoch: None,
            pending_source: None,
            frame: None,
            document_origin: None,
            texture: None,
            viewport: None,
            last_pointer_position: None,
            primary_pointer_pressed: false,
            focused: false,
            pointer_over: false,
            last_display_rect: None,
            error: Some(error),
            frame_update_deadline: None,
            pending_navigation_urls: VecDeque::new(),
            navigation_history: VecDeque::new(),
        }
    }
}
