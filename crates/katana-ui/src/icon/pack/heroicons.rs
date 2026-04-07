use super::{IconPackContract, IconPackManifest, LicenseMetadata, RenderPolicy};
use crate::icon::Icon;

pub struct HeroiconsPack;

impl IconPackContract for HeroiconsPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "heroicons",
            display_name: "Heroicons",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://heroicons.com/",
                license_text: None,
            },
        }
    }

    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        impl_icon_pack_match!("heroicons", icon)
    }
}
