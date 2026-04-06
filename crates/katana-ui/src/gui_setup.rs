use katana_core::plugin::{ExtensionPoint, PLUGIN_API_VERSION, PluginMeta, PluginRegistry};

pub struct GuiSetupOps;

impl GuiSetupOps {
    pub fn setup_fonts(ctx: &egui::Context) {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
        Self::setup_fonts_from_preset(ctx, preset);
    }

    pub fn setup_fonts_from_preset(
        ctx: &egui::Context,
        preset: &katana_core::markdown::color_preset::DiagramColorPreset,
    ) {
        katana_ui::font_loader::SystemFontLoader::setup_fonts(ctx, preset, None, None);
    }

    pub fn setup_fonts_with_candidates(ctx: &egui::Context, candidates: &[&str]) {
        let normalized = Self::build_font_definitions(candidates, &[], &[]);
        ctx.set_fonts(normalized.into_inner());

        #[cfg(debug_assertions)]
        ctx.global_style_mut(|style| {
            style.debug.debug_on_hover = false;
            style.debug.show_expand_width = false;
            style.debug.show_expand_height = false;
            style.debug.show_widget_hits = false;
        });
    }

    pub fn build_font_definitions(
        proportional_candidates: &[&str],
        monospace_candidates: &[&str],
        emoji_candidates: &[&str],
    ) -> katana_ui::font_loader::NormalizeFonts {
        katana_ui::font_loader::SystemFontLoader::build_font_definitions(
            proportional_candidates,
            monospace_candidates,
            emoji_candidates,
            None,
            None,
        )
    }

    pub fn load_first_font(candidates: &[&str]) -> Option<(String, Vec<u8>)> {
        for &path in candidates {
            let Ok(data) = std::fs::read(path) else {
                continue;
            };
            let name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cjk_font")
                .to_string();
            return Some((name, data));
        }
        None
    }

    pub fn register_builtin_plugins(registry: &mut PluginRegistry) {
        registry.register(
            PluginMeta {
                id: "builtin-mermaid-renderer".to_string(),
                name: "Built-in Mermaid Renderer".to_string(),
                api_version: PLUGIN_API_VERSION,
                extension_points: vec![ExtensionPoint::RendererEnhancement],
            },
            || Ok(()), /* WHY: Renderer logic is wired directly in the markdown pipeline. */
        );

        registry.register(
            PluginMeta {
                id: "builtin-plantuml-renderer".to_string(),
                name: "Built-in PlantUML Renderer".to_string(),
                api_version: PLUGIN_API_VERSION,
                extension_points: vec![ExtensionPoint::RendererEnhancement],
            },
            || Ok(()),
        );

        registry.register(
            PluginMeta {
                id: "builtin-drawio-renderer".to_string(),
                name: "Built-in Draw.io Renderer".to_string(),
                api_version: PLUGIN_API_VERSION,
                extension_points: vec![ExtensionPoint::RendererEnhancement],
            },
            || Ok(()),
        );
    }
}
