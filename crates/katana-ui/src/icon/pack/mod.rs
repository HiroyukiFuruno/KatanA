use crate::icon::Icon;

#[rustfmt::skip]
macro_rules! impl_icon_pack_match {
    ($dir:expr, $icon:expr) => {
        match $icon {
            crate::icon::Icon::Dot => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/dot", ".svg"))),
            crate::icon::Icon::Close => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/close", ".svg"))),
            crate::icon::Icon::Remove => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/remove", ".svg"))),
            crate::icon::Icon::Plus => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/plus", ".svg"))),
            crate::icon::Icon::Minus => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/minus", ".svg"))),
            crate::icon::Icon::CloseModal => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/close_modal", ".svg"))),
            crate::icon::Icon::Pin => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/pin", ".svg"))),
            crate::icon::Icon::Filter => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/filter", ".svg"))),
            crate::icon::Icon::Copy => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/copy", ".svg"))),
            crate::icon::Icon::ExpandAll => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/expand_all", ".svg"))),
            crate::icon::Icon::CollapseAll => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/collapse_all", ".svg"))),
            crate::icon::Icon::Search => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/search", ".svg"))),
            crate::icon::Icon::Settings => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "ui/settings", ".svg"))),
            crate::icon::Icon::ChevronLeft => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/chevron_left", ".svg"))),
            crate::icon::Icon::ChevronRight => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/chevron_right", ".svg"))),
            crate::icon::Icon::ChevronDown => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/chevron_down", ".svg"))),
            crate::icon::Icon::TriangleDown => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/triangle_down", ".svg"))),
            crate::icon::Icon::TriangleLeft => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/triangle_left", ".svg"))),
            crate::icon::Icon::TriangleRight => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/triangle_right", ".svg"))),
            crate::icon::Icon::ArrowUp => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/arrow_up", ".svg"))),
            crate::icon::Icon::ArrowDown => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/arrow_down", ".svg"))),
            crate::icon::Icon::ArrowLeft => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/arrow_left", ".svg"))),
            crate::icon::Icon::ArrowRight => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/arrow_right", ".svg"))),
            crate::icon::Icon::Toc => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "navigation/toc", ".svg"))),
            crate::icon::Icon::PanUp => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/pan_up", ".svg"))),
            crate::icon::Icon::PanDown => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/pan_down", ".svg"))),
            crate::icon::Icon::PanLeft => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/pan_left", ".svg"))),
            crate::icon::Icon::PanRight => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/pan_right", ".svg"))),
            crate::icon::Icon::ZoomIn => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/zoom_in", ".svg"))),
            crate::icon::Icon::ZoomOut => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/zoom_out", ".svg"))),
            crate::icon::Icon::ResetView => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/reset_view", ".svg"))),
            crate::icon::Icon::Fullscreen => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "view/fullscreen", ".svg"))),
            crate::icon::Icon::SplitVertical => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "layout/split_vertical", ".svg"))),
            crate::icon::Icon::SplitHorizontal => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "layout/split_horizontal", ".svg"))),
            crate::icon::Icon::SwapHorizontal => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "layout/swap_horizontal", ".svg"))),
            crate::icon::Icon::SwapVertical => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "layout/swap_vertical", ".svg"))),
            crate::icon::Icon::Preview => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "layout/preview", ".svg"))),
            crate::icon::Icon::Info => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "status/info", ".svg"))),
            crate::icon::Icon::Success => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "status/success", ".svg"))),
            crate::icon::Icon::Warning => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "status/warning", ".svg"))),
            crate::icon::Icon::Error => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "status/error", ".svg"))),
            crate::icon::Icon::Explorer => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "files/explorer", ".svg"))),
            crate::icon::Icon::Document => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "files/document", ".svg"))),
            crate::icon::Icon::FolderOpen => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "files/folder_open", ".svg"))),
            crate::icon::Icon::FolderClosed => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "files/folder_closed", ".svg"))),
            crate::icon::Icon::Markdown => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "files/markdown", ".svg"))),
            crate::icon::Icon::Refresh => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/refresh", ".svg"))),
            crate::icon::Icon::ExternalLink => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/external_link", ".svg"))),
            crate::icon::Icon::Export => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/export", ".svg"))),
            crate::icon::Icon::Github => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/github", ".svg"))),
            crate::icon::Icon::Heart => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/heart", ".svg"))),
            crate::icon::Icon::Bug => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/bug", ".svg"))),
            crate::icon::Icon::Action => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/action", ".svg"))),
            crate::icon::Icon::Recent => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/recent", ".svg"))),
            crate::icon::Icon::History => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/history", ".svg"))),
            crate::icon::Icon::Rocket => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/rocket", ".svg"))),
            crate::icon::Icon::Download => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/download", ".svg"))),
            crate::icon::Icon::Hourglass => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/hourglass", ".svg"))),
            crate::icon::Icon::More => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "action/more", ".svg"))),
            crate::icon::Icon::MatchCase => Some(include_bytes!("../../../../../assets/icons/system/match-case.svg")),
            crate::icon::Icon::WholeWord => Some(include_bytes!("../../../../../assets/icons/system/whole-word.svg")),
            crate::icon::Icon::UseRegex => Some(include_bytes!("../../../../../assets/icons/system/use-regex.svg")),
            crate::icon::Icon::Help => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "status/help", ".svg"))),
            crate::icon::Icon::Tools => Some(include_bytes!(concat!("../../../../../assets/icons/", $dir, "/", "system/tools", ".svg"))),
        }
    };
}

mod feather;
mod heroicons;
mod katana;
mod lucide;
mod material_symbols;
mod tabler_icons;

pub use feather::*;
pub use heroicons::*;
pub use katana::*;
pub use lucide::*;
pub use material_symbols::*;
pub use tabler_icons::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RenderPolicy {
    TintedMonochrome,
    NativeColor,
}

#[derive(Debug, Clone)]
pub struct LicenseMetadata {
    pub name: &'static str,
    pub source_url: &'static str,
    pub license_text: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct IconPackManifest {
    pub id: &'static str,
    pub display_name: &'static str,
    pub render_policy: RenderPolicy,
    pub license: LicenseMetadata,
}

pub trait IconPackContract {
    fn manifest(&self) -> IconPackManifest;
    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]>;

    fn coverage_table(&self) -> Vec<(Icon, bool)> {
        crate::icon::ALL_ICONS
            .iter()
            .map(|&icon| (icon, self.get_asset(icon).is_some()))
            .collect()
    }

    fn completeness_ratio(&self) -> f32 {
        let coverage = self.coverage_table();
        let total = coverage.len();
        if total == 0 {
            return 1.0;
        }
        let provided = coverage.iter().filter(|(_, provided)| *provided).count();
        provided as f32 / total as f32
    }
}

pub const AVAILABLE_PACKS: &[&dyn IconPackContract] = &[
    &KatanaIconPack,
    &MaterialSymbolsPack,
    &LucidePack,
    &TablerIconsPack,
    &HeroiconsPack,
    &FeatherPack,
];
