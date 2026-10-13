use super::{IconPackContract, IconPackManifest, LicenseMetadata, RenderPolicy};
use crate::icon::Icon;

pub struct LucidePack;

impl IconPackContract for LucidePack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "lucide",
            display_name: "Lucide",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "ISC",
                source_url: "https://lucide.dev/",
                license_text: None,
            },
        }
    }

    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        impl_icon_pack_match!("lucide", icon)
    }
}
