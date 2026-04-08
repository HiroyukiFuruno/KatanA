use super::{ALL_ICONS, pack, types::IconRegistry};

#[derive(Clone, Copy)]
pub struct ActiveRenderPolicy(pub pack::RenderPolicy);

#[derive(Clone)]
pub struct ActiveOverrides(
    pub std::collections::HashMap<String, katana_platform::settings::types::icon::IconOverride>,
);

impl IconRegistry {
    pub fn install(ctx: &egui::Context) {
        Self::install_pack(ctx, &pack::KatanaIconPack, &Default::default());
    }

    pub fn install_pack_by_id(
        ctx: &egui::Context,
        pack_id: &str,
        settings: &katana_platform::settings::types::icon::IconSettings,
    ) {
        let pack = pack::AVAILABLE_PACKS
            .iter()
            .find(|p| p.manifest().id == pack_id)
            .copied()
            .unwrap_or(&pack::KatanaIconPack);

        Self::install_pack(ctx, pack, settings);
    }

    pub fn install_pack(
        ctx: &egui::Context,
        default_pack: &dyn pack::IconPackContract,
        settings: &katana_platform::settings::types::icon::IconSettings,
    ) {
        /* WHY: Clear previous textures and byte caches before registering the new pack */
        ctx.forget_all_images();

        let fallback_pack = pack::KatanaIconPack;

        for icon in ALL_ICONS {
            let target_pack = settings
                .get_override(icon.name())
                .and_then(|ov| ov.vendor.as_ref())
                .and_then(|vid| {
                    pack::AVAILABLE_PACKS
                        .iter()
                        .find(|p| p.manifest().id == vid)
                        .copied()
                })
                .unwrap_or(default_pack);

            let bytes = target_pack
                .get_asset(*icon)
                .or_else(|| pack::IconPackContract::get_asset(&fallback_pack, *icon))
                .expect("KatanaIconPack must provide all icons");

            ctx.include_bytes(icon.uri(), bytes);
        }

        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("katana_icon_render_policy"),
                ActiveRenderPolicy(default_pack.manifest().render_policy),
            );
            d.insert_temp(
                egui::Id::new("katana_icon_overrides"),
                ActiveOverrides(settings.active_overrides.clone()),
            );
        });
    }

    pub fn get_render_policy(ctx: &egui::Context) -> pack::RenderPolicy {
        ctx.data(|d| {
            d.get_temp::<ActiveRenderPolicy>(egui::Id::new("katana_icon_render_policy"))
                .map(|p| p.0)
                .unwrap_or(pack::RenderPolicy::TintedMonochrome)
        })
    }

    pub fn get_active_overrides(ctx: &egui::Context) -> Option<ActiveOverrides> {
        ctx.data(|d| d.get_temp::<ActiveOverrides>(egui::Id::new("katana_icon_overrides")))
    }
}
