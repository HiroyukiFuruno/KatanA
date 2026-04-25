use super::{IconPackContract, IconPackManifest, LicenseMetadata, RenderPolicy};
use crate::icon::Icon;

pub struct MaterialSymbolsPack;

impl IconPackContract for MaterialSymbolsPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "material-symbols",
            display_name: "MaterialSymbols",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "Apache License 2.0",
                source_url: "https://fonts.google.com/icons",
                license_text: None,
            },
        }
    }

    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        match icon {
            Icon::Bold
            | Icon::Italic
            | Icon::Strikethrough
            | Icon::Code
            | Icon::Heading
            | Icon::List
            | Icon::ListOrdered
            | Icon::Quote => None,
            _ => impl_icon_pack_match!("material-symbols", icon),
        }
    }
}
