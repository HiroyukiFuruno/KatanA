#![allow(clippy::useless_vec)]
#![allow(unsafe_op_in_unsafe_fn)]
#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::cognitive_complexity
)]

#[cfg(not(test))]
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;
#[cfg(not(test))]
use katana_platform::{JsonFileRepository, SettingsService};
#[cfg(not(test))]
use katana_ui::app_state::AppState;
#[cfg(not(test))]
use katana_ui::shell::KatanaApp;

mod locale_detection;
#[cfg(not(test))]
mod window_setup;
#[cfg(not(test))]
use locale_detection::detect_initial_language;
#[cfg(not(test))]
use window_setup::{initial_window_size, load_icon, min_window_size};

#[cfg(not(test))]
fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "katana_ui=info,katana_core=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting KatanA");

    #[cfg(target_os = "macos")]
    unsafe {
        katana_ui::native_menu::NativeMenuOps::set_process_name();
    }

    let ai_registry = AiProviderRegistry::new();

    let mut plugin_registry = PluginRegistry::new();
    GuiSetupOps::register_builtin_plugins(&mut plugin_registry);

    let repo = JsonFileRepository::with_default_path();
    let mut settings = SettingsService::new(Box::new(repo));

    settings.apply_os_default_theme();
    settings.apply_os_default_language(detect_initial_language());

    let saved_language = settings.settings().language.clone();
    let saved_icon_pack = settings.settings().theme.icon_pack.clone();
    let saved_workspace = settings.settings().workspace.last_workspace.clone();

    let cache = std::sync::Arc::new(katana_platform::DefaultCacheService::default());
    let state = AppState::new(ai_registry, plugin_registry, settings, cache);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("KatanA")
            .with_icon(load_icon())
            .with_inner_size(initial_window_size())
            .with_min_inner_size(min_window_size())
            .with_maximized(true),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "KatanA",
        native_options,
        Box::new(|cc| {
            GuiSetupOps::setup_fonts(&cc.egui_ctx);
            katana_ui::svg_loader::KatanaSvgLoader::install(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            katana_ui::icon::IconRegistry::install_pack_by_id(&cc.egui_ctx, &saved_icon_pack);

            #[cfg(target_os = "macos")]
            unsafe {
                katana_ui::native_menu::NativeMenuOps::setup();
                let png_bytes = include_bytes!("../../../assets/icon.iconset/icon_512x512.png");
                katana_ui::native_menu::NativeMenuOps::set_app_icon_png(
                    png_bytes.as_ptr(),
                    png_bytes.len(),
                );
            }

            katana_ui::i18n::I18nOps::set_language(&saved_language);
            katana_ui::shell_ui::ShellUiOps::update_native_menu_strings_from_i18n();

            let mut app = KatanaApp::new(state);

            let icon_png = include_bytes!("../../../assets/icon.iconset/icon_128x128.png");
            match image::load_from_memory(icon_png) {
                Ok(icon_image) => {
                    let rgba = icon_image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.into_raw();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    let texture = cc.egui_ctx.load_texture(
                        "about_icon",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    app.about_icon = Some(texture);
                }
                Err(e) => tracing::warn!("Failed to load about icon from memory: {}", e),
            }

            if let Some(ws_path) = saved_workspace {
                let path = std::path::PathBuf::from(&ws_path);
                if path.is_dir() {
                    app.trigger_action(katana_ui::app_state::AppAction::OpenWorkspace(path));
                    tracing::info!("Restored workspace: {ws_path}");
                }
            }

            Ok(Box::new(app))
        }),
    )
}

pub mod gui_setup;
use gui_setup::GuiSetupOps;

#[cfg(test)]
mod tests {
    use super::*;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[test]
    fn test_load_first_font_not_found() {
        let candidates = vec!["/invalid/path/to/never/found/font.ttc"];
        let result = GuiSetupOps::load_first_font(&candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_load_first_font_found() {
        let candidates = vec![
            "/System/Library/Fonts/AquaKana.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ];
        let result = GuiSetupOps::load_first_font(&candidates);
        if let Some((name, data)) = result {
            assert!(!name.is_empty());
            assert!(!data.is_empty());
        }
    }

    #[test]
    fn test_setup_fonts_with_cjk() {
        init_tracing();
        let ctx = egui::Context::default();
        GuiSetupOps::setup_fonts(&ctx);
    }

    #[test]
    fn test_setup_fonts_without_cjk() {
        init_tracing();
        let ctx = egui::Context::default();
        GuiSetupOps::setup_fonts_with_candidates(&ctx, &["/nonexistent/font.ttc"]);
    }

    #[test]
    fn test_register_builtin_plugins() {
        init_tracing();
        let mut registry = PluginRegistry::new();
        GuiSetupOps::register_builtin_plugins(&mut registry);
        assert_eq!(registry.active_count(), 3);
    }

    #[test]
    fn test_install_image_loaders_does_not_panic() {
        let ctx = egui::Context::default();
        katana_ui::svg_loader::KatanaSvgLoader::install(&ctx);
        assert!(ctx.is_loader_installed(katana_ui::svg_loader::KatanaSvgLoader::ID));
    }

    const PROP_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/\u{30d2}\u{30e9}\u{30ae}\u{30ce}\u{89d2}\u{30b4}\u{30b7}\u{30c3}\u{30af} W3.ttc",
        "/System/Library/Fonts/AquaKana.ttc",
    ];

    const MONO_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Monaco.ttf",
    ];

    #[test]
    fn test_proportional_font_is_primary_in_proportional_family() {
        init_tracing();
        if GuiSetupOps::load_first_font(PROP_CANDIDATES).is_none() {
            return;
        }
        let fonts = GuiSetupOps::build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, &[]);
        let proportional = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        let loaded_name = GuiSetupOps::load_first_font(PROP_CANDIDATES).unwrap().0;
        assert_eq!(
            proportional.first().unwrap(),
            &loaded_name,
            "CJK font SHOULD be at position 0 to dictate proper row height and fix jitter"
        );
    }

    #[test]
    fn test_monospace_font_is_primary_in_monospace_family() {
        init_tracing();
        if GuiSetupOps::load_first_font(MONO_CANDIDATES).is_none() {
            return;
        }
        let fonts = GuiSetupOps::build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, &[]);
        let monospace = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let mono_name = GuiSetupOps::load_first_font(MONO_CANDIDATES).unwrap().0;
        assert_eq!(
            monospace.first().unwrap(),
            &mono_name,
            "Monospace CJK font SHOULD be at position 0 to provide correct line height"
        );
    }

    #[test]
    fn test_proportional_font_is_cjk_fallback_in_monospace() {
        init_tracing();
        if GuiSetupOps::load_first_font(PROP_CANDIDATES).is_none()
            || GuiSetupOps::load_first_font(MONO_CANDIDATES).is_none()
        {
            return;
        }
        let fonts = GuiSetupOps::build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, &[]);
        let monospace = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let prop_name = GuiSetupOps::load_first_font(PROP_CANDIDATES).unwrap().0;
        let mono_fallback_name = format!("{}_mono_fallback", prop_name);
        assert!(
            monospace.contains(&mono_fallback_name),
            "Proportional font should be in Monospace family as CJK fallback"
        );
        let mono_name = GuiSetupOps::load_first_font(MONO_CANDIDATES).unwrap().0;
        let mono_pos = monospace.iter().position(|n| n == &mono_name).unwrap();
        let prop_pos = monospace
            .iter()
            .position(|n| n == &mono_fallback_name)
            .unwrap();
        assert!(
            mono_pos < prop_pos,
            "Monospace font must appear before proportional (CJK fallback)"
        );
    }

    #[test]
    fn test_build_font_definitions_without_candidates_returns_defaults() {
        init_tracing();
        let fonts = GuiSetupOps::build_font_definitions(&["/nonexistent/font.ttc"], &[], &[]);
        let proportional = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        assert!(
            !proportional.is_empty(),
            "Proportional family should have default egui fonts"
        );
    }

    #[test]
    fn test_setup_fonts_from_preset_does_not_panic() {
        init_tracing();
        let ctx = egui::Context::default();
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current();
        GuiSetupOps::setup_fonts_from_preset(&ctx, preset);
    }

    #[test]
    fn test_preset_syntax_themes_are_valid_identifiers() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        assert!(
            !preset.syntax_theme_dark.is_empty(),
            "syntax_theme_dark must not be empty"
        );
        assert!(
            !preset.syntax_theme_light.is_empty(),
            "syntax_theme_light must not be empty"
        );
    }

    #[test]
    fn test_preset_preview_text_is_valid_hex_color() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        let parsed = DiagramColorPreset::parse_hex_rgb(preset.preview_text);
        assert!(
            parsed.is_some(),
            "preview_text '{}' must be a valid #RRGGBB hex",
            preset.preview_text
        );
    }

    #[test]
    fn test_preset_dark_and_light_have_different_preview_text() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        assert_ne!(
            DiagramColorPreset::dark().preview_text,
            DiagramColorPreset::light().preview_text,
            "DARK and LIGHT presets should have different preview text colors"
        );
    }

    const EMOJI_CANDIDATES: &[&str] = &[
        "/System/Library/Fonts/Apple Color Emoji.ttc",
        "C:/Windows/Fonts/seguiemj.ttf",
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
    ];

    #[test]
    #[cfg(target_os = "macos")]
    fn test_emoji_font_available_on_macos() {
        let result = GuiSetupOps::load_first_font(EMOJI_CANDIDATES);
        assert!(
            result.is_some(),
            "Apple Color Emoji font should be available on macOS"
        );
    }

    #[test]
    fn test_emoji_font_is_not_in_proportional_family() {
        init_tracing();
        if GuiSetupOps::load_first_font(EMOJI_CANDIDATES).is_none() {
            return;
        }
        let fonts =
            GuiSetupOps::build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, EMOJI_CANDIDATES);
        let proportional = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("Proportional family missing");
        let emoji_name = GuiSetupOps::load_first_font(EMOJI_CANDIDATES).unwrap().0;
        assert!(
            !proportional.contains(&emoji_name),
            "Preview emoji should not replace UI fallback fonts in Proportional family"
        );
    }

    #[test]
    fn test_emoji_font_is_not_in_monospace_family() {
        init_tracing();
        if GuiSetupOps::load_first_font(EMOJI_CANDIDATES).is_none() {
            return;
        }
        let fonts =
            GuiSetupOps::build_font_definitions(PROP_CANDIDATES, MONO_CANDIDATES, EMOJI_CANDIDATES);
        let monospace = fonts
            .fonts()
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("Monospace family missing");
        let emoji_name = GuiSetupOps::load_first_font(EMOJI_CANDIDATES).unwrap().0;
        assert!(
            !monospace.contains(&emoji_name),
            "Preview emoji should not replace UI fallback fonts in Monospace family"
        );
    }

    #[test]
    fn test_preset_has_emoji_font_candidates() {
        use katana_core::markdown::color_preset::DiagramColorPreset;
        let preset = DiagramColorPreset::current();
        assert!(
            !preset.emoji_font_candidates.is_empty(),
            "Preset must have at least one emoji font candidate"
        );
    }
}
