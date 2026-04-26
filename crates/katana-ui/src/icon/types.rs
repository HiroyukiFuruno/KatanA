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
        }

        pub const ALL_ICONS: &[Icon] = &[
            $( Icon::$variant, )+
        ];
    };
}

define_icons! {
    /* WHY: ui/ */
    Dot             => "ui/dot",
    Close           => "ui/close",
    Remove          => "ui/remove",
    Plus            => "ui/plus",
    Minus           => "ui/minus",
    CloseModal      => "ui/close_modal",
    Pin             => "ui/pin",
    Filter          => "ui/filter",
    MatchCase       => "../system/match-case",
    WholeWord       => "../system/whole-word",
    UseRegex        => "../system/use-regex",
    Copy            => "ui/copy",
    ExpandAll       => "ui/expand_all",
    CollapseAll     => "ui/collapse_all",
    Search          => "ui/search",
    Settings        => "ui/settings",
    Help            => "status/help",
    /* WHY: navigation/ */
    ChevronLeft     => "navigation/chevron_left",
    ChevronRight    => "navigation/chevron_right",
    ChevronDown     => "navigation/chevron_down",
    TriangleDown    => "navigation/triangle_down",
    TriangleLeft    => "navigation/triangle_left",
    TriangleRight   => "navigation/triangle_right",
    ArrowUp         => "navigation/arrow_up",
    ArrowDown       => "navigation/arrow_down",
    ArrowLeft       => "navigation/arrow_left",
    ArrowRight      => "navigation/arrow_right",
    Toc             => "navigation/toc",
    /* WHY: view/ */
    PanUp           => "view/pan_up",
    PanDown         => "view/pan_down",
    PanLeft         => "view/pan_left",
    PanRight        => "view/pan_right",
    ZoomIn          => "view/zoom_in",
    ZoomOut         => "view/zoom_out",
    ResetView       => "view/reset_view",
    Fullscreen      => "view/fullscreen",
    /* WHY: layout/ */
    SplitVertical   => "layout/split_vertical",
    SplitHorizontal => "layout/split_horizontal",
    SwapHorizontal  => "layout/swap_horizontal",
    SwapVertical    => "layout/swap_vertical",
    Preview         => "layout/preview",
    /* WHY: status/ */
    Info            => "status/info",
    Success         => "status/success",
    Warning         => "status/warning",
    Error           => "status/error",
    /* WHY: files/ */
    Explorer        => "files/explorer",
    Document        => "files/document",
    FilePlus        => "files/file_plus",
    FolderOpen      => "files/folder_open",
    FolderClosed    => "files/folder_closed",
    FolderPlus      => "files/folder_plus",
    Markdown        => "files/markdown",
    Html            => "files/html",
    Pdf             => "files/pdf",
    Image           => "files/image",
    /* WHY: system/ */
    Refresh         => "system/refresh",
    ExternalLink    => "system/external_link",
    Export          => "system/export",
    Github          => "system/github",
    Heart           => "system/heart",
    Bug             => "system/bug",
    Action          => "system/action",
    Recent          => "system/recent",
    History         => "system/history",
    Rocket          => "system/rocket",
    Download        => "system/download",
    Hourglass       => "system/hourglass",
    Tools           => "system/tools",
    MacCmd          => "../system/cmd",
    MacCtrl         => "../system/ctrl",
    MacShift        => "../system/shift",
    MacAlt          => "../system/alt",
    MacWin          => "../system/win",
    /* WHY: action/ */
    #[allow(dead_code)]
    More            => "action/more",
    Edit            => "action/edit",
    Bold            => "action/bold",
    Italic          => "action/italic",
    Strikethrough   => "action/strikethrough",
    Code            => "action/code",
    Heading         => "action/heading",
    List            => "action/list",
    ListOrdered     => "action/list-ordered",
    Quote           => "action/quote",
    /* WHY: Diagnostic gutter icon — 💡 LightBulb for lint hints. */
    LightBulb           => "action/light_bulb",
    CircleFilled        => "../system/circle-filled",
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    Small,
    Medium,
    Large,
}

pub struct IconOps;

pub struct IconRegistry;
