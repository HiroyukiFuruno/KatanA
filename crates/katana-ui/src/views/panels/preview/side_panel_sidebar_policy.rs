use super::side_panels::PreviewSidePanels;

#[derive(Clone, Copy)]
pub(super) struct SidebarAvailability {
    pub(super) toc: bool,
    pub(super) refresh: bool,
    pub(super) search: bool,
    pub(super) export: bool,
    pub(super) story: bool,
    pub(super) tools: bool,
    pub(super) info: bool,
}

impl PreviewSidePanels<'_> {
    pub(super) fn sidebar_availability(&self) -> SidebarAvailability {
        let settings = super::PreviewMenuAvailability::settings_for_state(&self.app.state);
        SidebarAvailability {
            toc: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Toc,
            )),
            refresh: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Refresh,
            )),
            search: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Search,
            )),
            export: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Export,
            )),
            story: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Story,
            )),
            tools: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Tools,
            )),
            info: settings.allows(super::DocumentOperation::PreviewMenu(
                super::PreviewMenu::Info,
            )),
        }
    }
}
