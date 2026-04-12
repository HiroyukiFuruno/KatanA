use super::{IconPackContract, IconPackManifest, LicenseMetadata, RenderPolicy};
use crate::icon::Icon;

pub struct KatanaIconPack;

impl IconPackContract for KatanaIconPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "katana",
            display_name: "KatanA (Default)",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://github.com/HiroyukiFuruno/KatanA",
                license_text: None,
            },
        }
    }

    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        Some(icon.svg_bytes())
    }
}
