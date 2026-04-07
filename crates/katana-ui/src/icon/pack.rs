use super::types::Icon;

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

/// The contract that each built-in pack must implement to resolve icon assets.
pub trait IconPackContract {
    fn manifest(&self) -> IconPackManifest;

    /// Attempts to load the SVG bytes for the active pack.
    /// Returns `None` if the specific asset is not available and should fallback.
    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]>;

    /// Generates a coverage table representing which icons are provided by this pack.
    fn coverage_table(&self) -> Vec<(Icon, bool)> {
        super::types::ALL_ICONS
            .iter()
            .map(|&icon| (icon, self.get_asset(icon).is_some()))
            .collect()
    }

    /// Calculate current coverage percentage.
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

/* WHY: 1.5 Selected Curated Packs */

pub struct KatanaIconPack;
impl IconPackContract for KatanaIconPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "katana",
            display_name: "KatanA (Default)",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://github.com/hiroyuki-furuno/katana",
                license_text: None,
            },
        }
    }
    fn get_asset(&self, icon: Icon) -> Option<&'static [u8]> {
        /* WHY: The `katana` pack is the definitive default, thus it must have an asset for every icon.
        It uses the same include_bytes! mapping from `types.rs`. */
        Some(icon.svg_bytes())
    }
}

pub struct MaterialSymbolsPack;
impl IconPackContract for MaterialSymbolsPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "material-symbols",
            display_name: "Material Symbols",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "Apache License 2.0",
                source_url: "https://fonts.google.com/icons",
                license_text: None,
            },
        }
    }
    fn get_asset(&self, _icon: Icon) -> Option<&'static [u8]> {
        None
    }
}

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
    fn get_asset(&self, _icon: Icon) -> Option<&'static [u8]> {
        None
    }
}

pub struct TablerIconsPack;
impl IconPackContract for TablerIconsPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "tabler-icons",
            display_name: "Tabler Icons",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://tabler.io/icons",
                license_text: None,
            },
        }
    }
    fn get_asset(&self, _icon: Icon) -> Option<&'static [u8]> {
        None
    }
}

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
    fn get_asset(&self, _icon: Icon) -> Option<&'static [u8]> {
        None
    }
}

pub struct FeatherPack;
impl IconPackContract for FeatherPack {
    fn manifest(&self) -> IconPackManifest {
        IconPackManifest {
            id: "feather",
            display_name: "Feather Icons",
            render_policy: RenderPolicy::TintedMonochrome,
            license: LicenseMetadata {
                name: "MIT",
                source_url: "https://feathericons.com/",
                license_text: None,
            },
        }
    }
    fn get_asset(&self, _icon: Icon) -> Option<&'static [u8]> {
        None
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
