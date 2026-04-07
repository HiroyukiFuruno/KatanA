use crate::elements::blockquote;
use egui::Ui;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Alert {
    pub accent_color: egui::Color32,
    pub icon: String,
    pub identifier: String,
    pub identifier_rendered: String,
}

pub fn alert_ui(alert: &Alert, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    blockquote(ui, alert.accent_color, |ui| {
        ui.vertical(|ui| {
            // 見出し上マージンの調整 (blockquoteのライン開始位置はy+5)
            // ラインの上端から8px下にタイトルが来るように調整
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.colored_label(alert.accent_color, &alert.icon);
                ui.add_space(4.0);
                ui.colored_label(alert.accent_color, &alert.identifier_rendered);
            });
            
            // 見出しと本文の間隔を適切に確保 (デフォルトのParagraphマージンを考慮して4px)
            ui.add_space(4.0);
            
            add_contents(ui);
            
            // ブロック下部の余白
            ui.add_space(8.0);
        });
    })
}

#[derive(Debug, Clone)]
pub struct AlertBundle {
    /// the key is `[!identifier]`
    alerts: HashMap<String, Alert>,
}

impl AlertBundle {
    pub fn from_alerts(alerts: Vec<Alert>) -> Self {
        let mut map = HashMap::with_capacity(alerts.len());
        for alert in alerts {
            // Store it the way it will be in text to make lookup easier
            map.insert(format!("[!{}]", alert.identifier), alert);
        }

        Self { alerts: map }
    }

    pub fn into_alerts(self) -> Vec<Alert> {
        // since the rendered field can be changed it is better to force creation of
        // a new bundle with from_alerts after a potential modification

        self.alerts.into_values().collect::<Vec<_>>()
    }

    pub fn empty() -> Self {
        AlertBundle {
            alerts: Default::default(),
        }
    }

    /// github flavoured markdown alerts
    /// `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]` and `[!CAUTION]`.
    ///
    /// This is used by default
    pub fn gfm() -> Self {
        Self::from_alerts(vec![
            Alert {
                accent_color: egui::Color32::from_rgb(10, 80, 210),
                icon: "ⓘ".to_string(),
                identifier: "NOTE".to_owned(),
                identifier_rendered: "Note".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(0, 130, 20),
                icon: "💡".to_string(),
                identifier: "TIP".to_owned(),
                identifier_rendered: "Tip".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(150, 30, 140),
                icon: "💬".to_string(),
                identifier: "IMPORTANT".to_owned(),
                identifier_rendered: "Important".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(200, 120, 0),
                icon: "⚠".to_string(),
                identifier: "WARNING".to_owned(),
                identifier_rendered: "Warning".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(220, 0, 0),
                icon: "🚫".to_string(), // Prohibited Sign Emoji
                identifier: "CAUTION".to_owned(),
                identifier_rendered: "Caution".to_owned(),
            },
        ])
    }

    /// See if the bundle contains no alerts
    pub fn is_empty(&self) -> bool {
        self.alerts.is_empty()
    }
}

pub fn try_get_alert<'a>(bundle: &'a AlertBundle, text: &str) -> Option<&'a Alert> {
    bundle.alerts.get(&text.to_uppercase())
}
