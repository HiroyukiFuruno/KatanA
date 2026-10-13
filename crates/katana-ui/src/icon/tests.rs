use super::*;

#[test]
fn icon_size_to_vec2_returns_correct_dimensions() {
    assert_eq!(
        IconSize::Small.to_vec2(),
        egui::vec2(IconSize::SMALL, IconSize::SMALL)
    );
    assert_eq!(
        IconSize::Medium.to_vec2(),
        egui::vec2(IconSize::MEDIUM, IconSize::MEDIUM)
    );
    assert_eq!(
        IconSize::Large.to_vec2(),
        egui::vec2(IconSize::LARGE, IconSize::LARGE)
    );
}

#[test]
fn icon_uri_follows_bytes_scheme() {
    assert_eq!(Icon::Refresh.uri(), "bytes://icon/system/refresh.svg");
    assert_eq!(
        Icon::ChevronLeft.uri(),
        "bytes://icon/navigation/chevron_left.svg"
    );
    assert_eq!(Icon::FilePlus.uri(), "bytes://icon/files/file_plus.svg");
    assert_eq!(Icon::FolderPlus.uri(), "bytes://icon/files/folder_plus.svg");
}

#[test]
fn try_from_emoji_maps_correctly() {
    assert_eq!(Icon::try_from_emoji('📄'), Some(Icon::Document));
    assert_eq!(Icon::try_from_emoji('📝'), Some(Icon::Markdown));
}

#[test]
fn katana_pack_provides_creation_icons() {
    use crate::icon::pack::IconPackContract;

    let pack = crate::icon::pack::KatanaIconPack;

    assert!(pack.get_asset(Icon::FilePlus).is_some());
    assert!(pack.get_asset(Icon::FolderPlus).is_some());
}
