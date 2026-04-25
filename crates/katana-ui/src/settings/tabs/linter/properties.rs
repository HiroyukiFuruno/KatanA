use eframe::egui;

pub(super) struct RulePropertiesOps;

impl RulePropertiesOps {
    pub(super) fn render_string_array_property(
        ui: &mut egui::Ui,
        meta: &katana_markdown_linter::rules::markdown::OfficialRuleMeta,
        prop: &katana_markdown_linter::rules::markdown::RuleProperty,
        config: &mut katana_markdown_linter::rules::markdown::config::MarkdownLintConfig,
        target_path: &std::path::Path,
    ) {
        let mut current_arr: Vec<String> = config
            .get_rule_property(meta.code, prop.key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| serde_json::from_str(prop.default_value).unwrap_or_default());

        let mut changed = false;

        crate::widgets::Accordion::new(
            ui.id().with(format!("{}_{}_arr", meta.code, prop.key)),
            prop.key.to_string(),
            |ui| {
                changed = crate::settings::tabs::LayoutTabOps::render_string_list_editor(
                    ui,
                    &mut current_arr,
                );
            },
        )
        .show(ui);

        if changed {
            let Ok(json_val) = serde_json::to_value(&current_arr) else {
                return;
            };
            super::properties_helpers::PropertiesHelpersOps::save_property(
                ui,
                meta,
                prop,
                json_val,
                config,
                target_path,
            );
        }
    }

    pub(super) fn render_singleline_property(
        ui: &mut egui::Ui,
        meta: &katana_markdown_linter::rules::markdown::OfficialRuleMeta,
        prop: &katana_markdown_linter::rules::markdown::RuleProperty,
        config: &mut katana_markdown_linter::rules::markdown::config::MarkdownLintConfig,
        target_path: &std::path::Path,
    ) {
        if prop.prop_type == katana_markdown_linter::rules::markdown::RulePropertyType::Boolean {
            let current_bool: bool = config
                .get_rule_property(meta.code, prop.key)
                .and_then(|v| v.as_bool())
                .unwrap_or_else(|| {
                    serde_json::from_str::<bool>(prop.default_value).unwrap_or(false)
                });

            let mut checked = current_bool;
            let mut changed_by_toggle = false;

            /* WHY: Use AlignCenter with interactive=true so the entire row acts like a
             * clickable native list item, and ToggleOps draws the toggle perfectly aligned to the right. */
            let row_response = crate::widgets::AlignCenter::new()
                .interactive(true)
                .left(|ui| ui.label(prop.key))
                .right(|ui| {
                    if crate::widgets::ToggleOps::switch(ui, &mut checked).changed() {
                        changed_by_toggle = true;
                    }
                    ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                })
                .show(ui);

            if row_response.clicked() && !changed_by_toggle {
                checked = !checked;
                changed_by_toggle = true;
            }

            if changed_by_toggle {
                let json_val = serde_json::Value::Bool(checked);
                super::properties_helpers::PropertiesHelpersOps::save_property(
                    ui,
                    meta,
                    prop,
                    json_val,
                    config,
                    target_path,
                );
            }
            return;
        }

        let mut current_val = config
            .get_rule_property(meta.code, prop.key)
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_else(|| {
                let dv = prop.default_value;
                serde_json::from_str::<String>(dv).unwrap_or_else(|_| dv.to_string())
            });

        crate::widgets::AlignCenter::new()
            .left(|ui| ui.label(prop.key))
            .right(|ui| {
                let mut changed = false;

                if let katana_markdown_linter::rules::markdown::RulePropertyType::Enum(opts) =
                    prop.prop_type
                {
                    const MAX_TOGGLE_OPTIONS: usize = 3;
                    if opts.len() <= MAX_TOGGLE_OPTIONS {
                        if ui
                            .add(crate::widgets::SegmentedStringToggle::new(
                                format!("{}_{}_seg", meta.code, prop.key),
                                opts,
                                &mut current_val,
                            ))
                            .changed()
                        {
                            changed = true;
                        }
                    } else if Self::render_combobox(
                        ui,
                        &format!("{}_{}_combo", meta.code, prop.key),
                        opts,
                        &mut current_val,
                    ) {
                        changed = true;
                    }
                } else {
                    const SINGLELINE_TEXT_WIDTH: f32 = 120.0;
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut current_val)
                                .desired_width(SINGLELINE_TEXT_WIDTH),
                        )
                        .lost_focus()
                    {
                        changed = true;
                    }
                }

                if changed {
                    let json_val = serde_json::from_str::<serde_json::Value>(&current_val)
                        .unwrap_or(serde_json::Value::String(current_val));
                    super::properties_helpers::PropertiesHelpersOps::save_property(
                        ui,
                        meta,
                        prop,
                        json_val,
                        config,
                        target_path,
                    );
                }
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);
    }

    pub(super) fn render_combobox(
        ui: &mut egui::Ui,
        combo_id: &str,
        opts: &[&str],
        current_val: &mut String,
    ) -> bool {
        let mut changed = false;
        const COMBO_BOX_WIDTH: f32 = 120.0;
        crate::widgets::StyledComboBox::new(combo_id, current_val.as_str())
            .width(COMBO_BOX_WIDTH)
            .show(ui, |ui| {
                for &opt in opts {
                    if ui
                        .add(
                            egui::Button::selectable(*current_val == opt, opt.to_string())
                                .frame_when_inactive(true),
                        )
                        .clicked()
                    {
                        *current_val = opt.to_string();
                        changed = true;
                    }
                }
            });
        changed
    }
}
