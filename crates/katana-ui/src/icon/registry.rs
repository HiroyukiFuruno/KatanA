use super::{ALL_ICONS, pack, types::IconRegistry};

#[derive(Clone, Copy)]
pub struct ActiveRenderPolicy(pub pack::RenderPolicy);

impl IconRegistry {
    pub fn install(ctx: &egui::Context) {
        Self::install_pack(ctx, &pack::KatanaIconPack);
    }

    pub fn install_pack_by_id(ctx: &egui::Context, pack_id: &str) {
        let pack = pack::AVAILABLE_PACKS
            .iter()
            .find(|p| p.manifest().id == pack_id)
            .copied()
            .unwrap_or(&pack::KatanaIconPack);

        Self::install_pack(ctx, pack);
    }

    pub fn install_pack(ctx: &egui::Context, pack: &dyn pack::IconPackContract) {
        /* WHY: Clear previous textures and byte caches before registering the new pack */
        ctx.forget_all_images();

        let fallback_pack = pack::KatanaIconPack;

        for icon in ALL_ICONS {
            let bytes = pack
                .get_asset(*icon)
                .or_else(|| pack::IconPackContract::get_asset(&fallback_pack, *icon))
                .expect("KatanaIconPack must provide all icons");

            ctx.include_bytes(icon.uri(), bytes);
        }

        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("katana_icon_render_policy"),
                ActiveRenderPolicy(pack.manifest().render_policy),
            )
        });
    }

    pub fn get_render_policy(ctx: &egui::Context) -> pack::RenderPolicy {
        ctx.data(|d| {
            d.get_temp::<ActiveRenderPolicy>(egui::Id::new("katana_icon_render_policy"))
                .map(|p| p.0)
                .unwrap_or(pack::RenderPolicy::TintedMonochrome)
        })
    }
}
