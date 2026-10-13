use super::{IconPackContract, IconPackManifest, LicenseMetadata, RenderPolicy};
use crate::icon::Icon;

pub struct TablerIconsPack;

impl IconPackContract for TablerIconsPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "tabler-icons",
            display_name: "TablerIcons",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://tabler.io/icons",
                license_text: None,
            },
        }
    }

    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        impl_icon_pack_match!("tabler-icons", icon)
    }
}
