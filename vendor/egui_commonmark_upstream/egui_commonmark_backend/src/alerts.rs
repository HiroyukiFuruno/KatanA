use crate::elements::blockquote;
use egui::Ui;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Alert {
    pub accent_color: egui::Color32,
    pub icon: char,
    pub identifier: String,
    pub identifier_rendered: String,
}

pub fn alert_ui(alert: &Alert, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    blockquote(ui, alert.accent_color, |ui| {
        ui.vertical(|ui| {
            // 見出しの上の余白。左のカラーライン（y+5で開始）より5px上から配置するため、10.0を設定
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.colored_label(alert.accent_color, alert.icon.to_string());
                ui.add_space(4.0);
                ui.colored_label(alert.accent_color, &alert.identifier_rendered);
            });
            // 見出しから本文の間の余白を極限まで縮める（add_contents内のParagraph由来のnewlineを相殺する場合あり）
            ui.add_space(-4.0);
        });
        
        add_contents(ui);
        
        ui.vertical(|ui| {
            ui.add_space(10.0);
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
                icon: 'ℹ',
                identifier: "NOTE".to_owned(),
                identifier_rendered: "Note".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(0, 130, 20),
                icon: '💡',
                identifier: "TIP".to_owned(),
                identifier_rendered: "Tip".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(150, 30, 140),
                icon: '💬',
                identifier: "IMPORTANT".to_owned(),
                identifier_rendered: "Important".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(200, 120, 0),
                icon: '⚠',
                identifier: "WARNING".to_owned(),
                identifier_rendered: "Warning".to_owned(),
            },
            Alert {
                accent_color: egui::Color32::from_rgb(220, 0, 0),
                icon: '❕', // ◯に！のアイコン
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
