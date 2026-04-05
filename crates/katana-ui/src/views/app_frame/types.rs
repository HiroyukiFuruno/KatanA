use crate::shell::KatanaApp;

pub(crate) struct MainPanels<'a> {
    pub app: &'a mut KatanaApp,
    pub theme_colors: &'a katana_platform::theme::ThemeColors,
}

pub(crate) struct WindowTitle<'a> {
    pub(crate) app: &'a mut KatanaApp,
}

pub(crate) struct TitleBar<'a> {
    pub(crate) app: &'a KatanaApp,
    pub(crate) theme_colors: &'a katana_platform::theme::ThemeColors,
}

pub(crate) struct WorkspaceSidebar<'a> {
    pub(crate) app: &'a mut KatanaApp,
}

pub(crate) struct WorkspaceSidebarItems;
pub(crate) struct WorkspaceSidebarDrag;

pub(crate) struct TabToolbar<'a> {
    pub(crate) app: &'a mut KatanaApp,
}

pub(crate) struct Breadcrumbs<'a> {
    pub(crate) app: &'a KatanaApp,
    pub(crate) rel: &'a str,
    pub(crate) ws_root: Option<&'a std::path::Path>,
}

pub(crate) struct CentralContent<'a> {
    pub(crate) app: &'a mut KatanaApp,
}

pub struct AppFrameOps;
