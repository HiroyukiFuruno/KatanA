macro_rules! define_icons {
    ( $( $(#[$meta:meta])* $variant:ident => $file:literal ),+ $(,)? ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $( $(#[$meta])* $variant, )+
        }

        impl Icon {
            pub const fn name(&self) -> &'static str {
                match self {
                    $( Self::$variant => $file, )+
                }
            }

            pub fn svg_bytes(&self) -> &'static [u8] {
                match self {
                    $( Self::$variant => include_bytes!(
                        concat!("../../../../assets/icons/", $file, ".svg")
                    ), )+
                }
            }
        }

        pub const ALL_ICONS: &[Icon] = &[
            $( Icon::$variant, )+
        ];
    };
}

define_icons! {
    // ui/
    Dot             => "ui/dot",
    Close           => "ui/close",
    Remove          => "ui/remove",
    Plus            => "ui/plus",
    Minus           => "ui/minus",
    CloseModal      => "ui/close_modal",
    Pin             => "ui/pin",
    Filter          => "ui/filter",
    Copy            => "ui/copy",
    ExpandAll       => "ui/expand_all",
    CollapseAll     => "ui/collapse_all",
    Search          => "ui/search",
    Settings        => "ui/settings",
    // navigation/
    ChevronLeft     => "navigation/chevron_left",
    ChevronRight    => "navigation/chevron_right",
    TriangleDown    => "navigation/triangle_down",
    TriangleLeft    => "navigation/triangle_left",
    TriangleRight   => "navigation/triangle_right",
    Toc             => "navigation/toc",
    // view/
    PanUp           => "view/pan_up",
    PanDown         => "view/pan_down",
    PanLeft         => "view/pan_left",
    PanRight        => "view/pan_right",
    ZoomIn          => "view/zoom_in",
    ZoomOut         => "view/zoom_out",
    ResetView       => "view/reset_view",
    Fullscreen      => "view/fullscreen",
    // layout/
    SplitVertical   => "layout/split_vertical",
    SplitHorizontal => "layout/split_horizontal",
    SwapHorizontal  => "layout/swap_horizontal",
    SwapVertical    => "layout/swap_vertical",
    Preview         => "layout/preview",
    // status/
    Info            => "status/info",
    Success         => "status/success",
    Warning         => "status/warning",
    Error           => "status/error",
    // files/
    Document        => "files/document",
    FolderOpen      => "files/folder_open",
    FolderClosed    => "files/folder_closed",
    Markdown        => "files/markdown",
    // system/
    Refresh         => "system/refresh",
    ExternalLink    => "system/external_link",
    Export          => "system/export",
    Github          => "system/github",
    Heart           => "system/heart",
    Bug             => "system/bug",
    Action          => "system/action",
    Recent          => "system/recent",
    Rocket          => "system/rocket",
    Download        => "system/download",
    Hourglass       => "system/hourglass",
    // action/
    #[allow(dead_code)]
    More            => "action/more",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Small,
    Medium,
    Large,
}

pub struct IconOps;

pub struct IconRegistry;
