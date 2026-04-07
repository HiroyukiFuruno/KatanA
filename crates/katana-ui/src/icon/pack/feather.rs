use super::{IconPackContract, IconPackManifest, LicenseMetadata, RenderPolicy};
use crate::icon::Icon;

pub struct FeatherPack;

impl IconPackContract for FeatherPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "feather",
            display_name: "Feather",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://feathericons.com/",
                license_text: None,
            },
        }
    }

    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        impl_icon_pack_match!("feather", icon)
    }
}
