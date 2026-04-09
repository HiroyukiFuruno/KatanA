use super::*;
#[cfg(target_os = "macos")]
use egui::FontData;
use egui::{FontDefinitions, FontId};
use katana_core::markdown::color_preset::DiagramColorPreset;
use std::fs;
#[cfg(target_os = "macos")]
use std::sync::Arc;
use tempfile::TempDir;

#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_NAME: &str = "Apple Color Emoji";

#[test]
fn test_normalize_fonts_is_normalized_state() {
    let raw = NormalizeFonts::new(FontDefinitions::default());
    assert!(!raw.is_normalized());
    let normalized = raw.normalize(&[]);
    assert!(normalized.is_normalized());
}

#[test]
fn test_normalize_fonts_double_normalize_is_noop() {
    let fonts = NormalizeFonts::new(FontDefinitions::default()).normalize(&[]);
    let family_before = fonts.fonts().families.clone();
    let fonts = fonts.normalize(&[]);
    assert_eq!(fonts.fonts().families, family_before);
}

#[test]
fn test_build_font_definitions_no_candidates() {
    let fonts = SystemFontLoader::build_font_definitions(&[], &[], &[], None, None).into_inner();
    assert!(!fonts.font_data.contains_key("cjk_font"));
    assert!(!fonts.font_data.contains_key("MyCustomFont"));
}

#[test]
fn test_build_font_definitions_includes_emoji_candidates_for_ui_families() {
    let tmp = TempDir::new().unwrap();
    let font_path = tmp.path().join("emoji.ttf");
    fs::write(&font_path, "").unwrap();
    let path_str = font_path.to_str().unwrap();

    let fonts =
        SystemFontLoader::build_font_definitions(&[], &[], &[path_str], None, None).into_inner();

    let emoji_name = "emoji";
    assert!(
        fonts.font_data.contains_key(emoji_name),
        "preview emoji should be included in global egui font families as fallbacks"
    );
    let prop_list = fonts.families.get(&FontFamily::Proportional).unwrap();
    assert!(prop_list.contains(&emoji_name.to_string()));
    let mono_list = fonts.families.get(&FontFamily::Monospace).unwrap();
    assert!(mono_list.contains(&emoji_name.to_string()));
}

#[test]
fn test_custom_font_injection() {
    let tmp = TempDir::new().unwrap();
    let custom_font_path = tmp.path().join("custom.ttf");
    fs::write(&custom_font_path, "").unwrap();
    let path_str = custom_font_path.to_str().unwrap();

    let fonts = SystemFontLoader::build_font_definitions(
        &[],
        &[],
        &[],
        Some(path_str),
        Some("MyCustomFont"),
    );

    assert!(fonts.fonts().font_data.contains_key("MyCustomFont"));
    let prop_list = fonts
        .fonts()
        .families
        .get(&FontFamily::Proportional)
        .unwrap();
    assert_eq!(prop_list.first().unwrap(), "MyCustomFont");
}

#[test]
fn test_custom_font_injection_invalid_path() {
    let fonts = SystemFontLoader::build_font_definitions(
        &[],
        &[],
        &[],
        Some("/path/does/not/exist.ttf"),
        Some("MyCustomFont"),
    );

    assert!(!fonts.fonts().font_data.contains_key("MyCustomFont"));
}

#[test]
#[cfg(target_os = "macos")]
fn test_apple_color_emoji_family_renders_directly() {
    let data = fs::read("/System/Library/Fonts/Apple Color Emoji.ttc").expect("apple emoji font");
    let mut fonts = FontDefinitions::empty();
    fonts.font_data.insert(
        APPLE_COLOR_EMOJI_FONT_NAME.to_string(),
        Arc::new(FontData::from_owned(data)),
    );
    fonts.families.insert(
        FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into()),
        vec![APPLE_COLOR_EMOJI_FONT_NAME.to_string()],
    );

    let ctx = Context::default();
    ctx.set_fonts(fonts);

    let mut glyph = None;
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                "🌍".to_owned(),
                FontId::new(24.0, FontFamily::Name(APPLE_COLOR_EMOJI_FONT_NAME.into())),
                crate::theme_bridge::WHITE,
            );
            glyph = galley
                .rows
                .first()
                .and_then(|row| row.glyphs.first())
                .copied();
        });
    });

    let glyph = glyph.expect("emoji glyph should be laid out");
    assert!(
        !glyph.uv_rect.is_nothing(),
        "Apple Color Emoji should rasterize a visible glyph when used directly"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_ui_font_setup_does_register_apple_color_emoji_globally() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let proportional = fonts
        .fonts()
        .families
        .get(&FontFamily::Proportional)
        .expect("proportional family");
    assert!(
        proportional.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
        "UI symbol glyphs should include emoji fonts"
    );

    let monospace = fonts
        .fonts()
        .families
        .get(&FontFamily::Monospace)
        .expect("monospace family");
    assert!(
        monospace.contains(&APPLE_COLOR_EMOJI_FONT_NAME.to_string()),
        "UI symbol glyphs should include emoji fonts"
    );
}

fn assert_font_jitter(context_name: &str, font_size: f32) {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let text = format!(
        "Katana — {} Lambda\u{30a2}\u{30c3}\u{30d7}\u{30c7}\u{30fc}\u{30c8}\u{624b}\u{9806}.md",
        context_name
    );
    let mut eng_glyph = None;
    let mut jpn_glyph = None;

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                text.clone(),
                FontId::proportional(font_size),
                crate::theme_bridge::WHITE,
            );
            eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'L').copied();
            jpn_glyph = galley.rows[0]
                .glyphs
                .iter()
                .find(|g| g.chr == '\u{30a2}')
                .copied();
        });
    });

    let eng_glyph = eng_glyph.expect("English char not found");
    let jpn_glyph = jpn_glyph.expect("Japanese char not found");

    assert_eq!(
        eng_glyph.pos.y, jpn_glyph.pos.y,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in {}: English 'L' y={} vs Japanese '\u{30a2}' y={}",
        context_name, eng_glyph.pos.y, jpn_glyph.pos.y
    );
}

#[test]
fn test_font_jitter_1_app_title() {
    assert_font_jitter("App Title", 20.0);
}

#[test]
fn test_font_jitter_2_workspace_dir() {
    assert_font_jitter("Workspace Dir", 14.0);
}

#[test]
fn test_font_jitter_3_workspace_file() {
    assert_font_jitter("Workspace File", 14.0);
}

#[test]
fn test_font_jitter_4_toc_heading() {
    assert_font_jitter("TOC Heading", 14.0);
}

#[test]
fn test_font_jitter_5_tab_name() {
    assert_font_jitter("Tab Name", 14.0);
}

#[test]
fn test_font_jitter_6_monospace() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let text =
        "cd infrastructures/tools/crypt-decrypt \u{306e}\u{5fa9}\u{53f7}\u{5316}".to_string();
    let mut eng_glyph = None;
    let mut jpn_glyph = None;

    let mut primitives = vec![];
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                text.clone(),
                FontId::monospace(14.0),
                crate::theme_bridge::WHITE,
            );
            eng_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'c').copied();
            jpn_glyph = galley.rows[0]
                .glyphs
                .iter()
                .find(|g| g.chr == '\u{5fa9}')
                .copied();

            let shapes = vec![egui::epaint::ClippedShape {
                clip_rect: egui::Rect::EVERYTHING,
                shape: egui::epaint::Shape::galley(
                    egui::Pos2::ZERO,
                    galley,
                    crate::theme_bridge::WHITE,
                ),
            }];
            primitives = ctx.tessellate(shapes, 1.0);
        });
    });

    let eng_glyph = eng_glyph.expect("English char not found");
    let jpn_glyph = jpn_glyph.expect("Japanese char not found");

    let mut eng_min_y = f32::INFINITY;
    let mut jpn_min_y = f32::INFINITY;

    if let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive {
        for v in &mesh.vertices {
            if v.pos.x >= eng_glyph.logical_rect().min.x
                && v.pos.x <= eng_glyph.logical_rect().max.x
            {
                eng_min_y = eng_min_y.min(v.pos.y);
            }
            if v.pos.x >= jpn_glyph.logical_rect().min.x
                && v.pos.x <= jpn_glyph.logical_rect().max.x
            {
                jpn_min_y = jpn_min_y.min(v.pos.y);
            }
        }
    }

    let diff = (eng_min_y - jpn_min_y).abs();
    assert!(
        diff <= 1.5,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in Monospace visual mesh: English 'c' y={} vs Japanese '\u{5fa9}' y={} (Diff: {})",
        eng_min_y,
        jpn_min_y,
        diff
    );
}

#[test]
fn test_font_jitter_7_codeblock_layoutjob() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let code_text = "# \u{5168}\u{4ef6}\u{5b9f}\u{884c}";

    let mut hash_glyph = None;
    let mut jp_glyph = None;
    let mut primitives = vec![];

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut job = egui::text::LayoutJob::default();
            job.append(
                code_text,
                0.0,
                egui::TextFormat::simple(
                    egui::TextStyle::Monospace.resolve(ui.style()),
                    crate::theme_bridge::WHITE,
                ),
            );
            job.wrap.max_width = 800.0;

            let galley = ui.fonts_mut(|f| f.layout_job(job));

            if let Some(row) = galley.rows.first() {
                for g in &row.glyphs {
                    if g.chr == '#' {
                        hash_glyph = Some(*g);
                    }
                    if g.chr == '\u{5168}' {
                        jp_glyph = Some(*g);
                    }
                }
            }

            let shapes = vec![egui::epaint::ClippedShape {
                clip_rect: egui::Rect::EVERYTHING,
                shape: egui::epaint::Shape::galley(
                    egui::Pos2::ZERO,
                    galley,
                    crate::theme_bridge::WHITE,
                ),
            }];
            primitives = ctx.tessellate(shapes, 1.0);
        });
    });

    let hash_glyph = hash_glyph.expect("'#' glyph not found");
    let jp_glyph = jp_glyph.expect("'\u{5168}' glyph not found");

    let mut hash_min_y = f32::INFINITY;
    let mut jp_min_y = f32::INFINITY;

    if let egui::epaint::Primitive::Mesh(mesh) = &primitives[0].primitive {
        for v in &mesh.vertices {
            if v.pos.x >= hash_glyph.logical_rect().min.x
                && v.pos.x <= hash_glyph.logical_rect().max.x
            {
                hash_min_y = hash_min_y.min(v.pos.y);
            }
            if v.pos.x >= jp_glyph.logical_rect().min.x && v.pos.x <= jp_glyph.logical_rect().max.x
            {
                jp_min_y = jp_min_y.min(v.pos.y);
            }
        }
    }

    let diff = (hash_min_y - jp_min_y).abs();
    assert!(
        diff <= 1.5,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in CodeBlock LayoutJob visual mesh: '#' y={} vs '\u{5168}' y={} (Diff: {}). \
         Mixed JP/EN text must share a common baseline.",
        hash_min_y,
        jp_min_y,
        diff
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_font_jitter_8_inline_code_cross_family() {
    use egui::TextStyle;
    use egui::text::{LayoutJob, TextFormat};

    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );
    let ctx = Context::default();
    ctx.set_fonts(fonts.into_inner());

    let mut prop_min_y = f32::INFINITY;
    let mut mono_min_y = f32::INFINITY;

    let mut primitives = vec![];
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut job = LayoutJob::default();

            let prop_format = TextFormat {
                font_id: TextStyle::Body.resolve(ui.style()),
                ..Default::default()
            };
            let mono_format = TextFormat {
                font_id: TextStyle::Monospace.resolve(ui.style()),
                ..Default::default()
            };

            job.append("\u{30a4}\u{30f3}\u{30b9}\u{30c8}\u{30fc}\u{30eb}\u{5f8c}\u{3001}", 0.0, prop_format.clone());
            job.append("mmdc", 0.0, mono_format);
            job.append(" \u{306f}\u{81ea}\u{52d5}\u{7684}\u{306b}\u{691c}\u{51fa}\u{3055}\u{308c}\u{307e}\u{3059}", 0.0, prop_format);

            let galley = ui.fonts_mut(|f| f.layout_job(job));

            let prop_glyph = galley.rows[0]
                .glyphs
                .iter()
                .find(|g| g.chr == '\u{30a4}')
                .copied();
            let mono_glyph = galley.rows[0].glyphs.iter().find(|g| g.chr == 'm').copied();

            if let (Some(pg), Some(mg)) = (prop_glyph, mono_glyph) {
                let shapes = vec![egui::epaint::ClippedShape {
                    clip_rect: egui::Rect::EVERYTHING,
                    shape: egui::epaint::Shape::galley(
                        egui::Pos2::ZERO,
                        galley,
                        crate::theme_bridge::WHITE,
                    ),
                }];
                primitives = ctx.tessellate(shapes, 1.0);

                if let Some(egui::epaint::Primitive::Mesh(mesh)) =
                    primitives.first().map(|p| &p.primitive)
                {
                    for v in &mesh.vertices {
                        if v.pos.x >= pg.logical_rect().min.x
                            && v.pos.x <= pg.logical_rect().max.x
                        {
                            prop_min_y = prop_min_y.min(v.pos.y);
                        }
                        if v.pos.x >= mg.logical_rect().min.x
                            && v.pos.x <= mg.logical_rect().max.x
                        {
                            mono_min_y = mono_min_y.min(v.pos.y);
                        }
                    }
                }
            }
        });
    });

    assert!(
        prop_min_y.is_finite() && mono_min_y.is_finite(),
        "Both glyphs must be found in mesh"
    );

    let diff = (prop_min_y - mono_min_y).abs();
    assert!(
        diff <= 10.0,
        "\u{30ac}\u{30bf}\u{30c4}\u{30ad} (Jitter) in inline code: Proportional '\u{30a4}' y={} vs Monospace 'm' y={} (Diff: {}). \
         Cross-family alignment is not enforced at font-level; handle at LayoutJob level.",
        prop_min_y,
        mono_min_y,
        diff
    );
}

#[test]
fn test_proportional_primary_y_offset_matches_constant() {
    let preset = DiagramColorPreset::current();
    let fonts = SystemFontLoader::build_font_definitions(
        &preset.proportional_font_candidates,
        &preset.monospace_font_candidates,
        &preset.emoji_font_candidates,
        None,
        None,
    );

    let prop_primary = fonts
        .fonts()
        .families
        .get(&FontFamily::Proportional)
        .and_then(|list| list.first())
        .cloned();

    if let Some(name) = prop_primary {
        let font_data = fonts.fonts().font_data.get(&name).expect("font data");
        let y_offset = font_data.tweak.y_offset_factor;
        let expected = super::PROPORTIONAL_Y_OFFSET_FACTOR;
        assert!(
            (y_offset - expected).abs() < 0.001,
            "Proportional primary font y_offset_factor must match PROPORTIONAL_Y_OFFSET_FACTOR ({}). Got: {}.",
            expected,
            y_offset
        );
    }
}
#[test]
fn test_setup_fonts_sets_fonts_loaded_flag() {
    let ctx = Context::default();
    let preset = DiagramColorPreset::current();
    SystemFontLoader::setup_fonts(&ctx, preset, None, None);

    let loaded = ctx.data(|d| d.get_temp::<bool>(egui::Id::new("katana_fonts_loaded")));
    assert!(
        loaded.is_some(),
        "katana_fonts_loaded flag must always be set"
    );
}
