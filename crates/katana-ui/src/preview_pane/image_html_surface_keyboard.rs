use super::HtmlBrowserSurface;
use eframe::egui;
use katana_document_viewer::browser_session::HtmlBrowserInput;

impl HtmlBrowserSurface {
    pub(super) fn forward_keyboard_events(&mut self, ui: &egui::Ui) {
        if !self.focused {
            return;
        }

        for event in ui.input(|input| input.events.clone()) {
            self.forward_keyboard_event(event);
        }
    }

    fn forward_keyboard_event(&mut self, event: egui::Event) {
        if let Some(input) = browser_keyboard_input(event) {
            self.dispatch(input);
        }
    }
}

fn browser_keyboard_input(event: egui::Event) -> Option<HtmlBrowserInput> {
    match event {
        egui::Event::Text(text)
        | egui::Event::Paste(text)
        | egui::Event::Ime(egui::ImeEvent::Commit(text))
            if !text.is_empty() =>
        {
            Some(HtmlBrowserInput::Text { text })
        }
        egui::Event::Key { key, pressed, .. } => {
            let key = format!("{key:?}");
            Some(if pressed {
                HtmlBrowserInput::KeyDown { key }
            } else {
                HtmlBrowserInput::KeyUp { key }
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::browser_keyboard_input;
    use eframe::egui;
    use katana_document_viewer::browser_session::HtmlBrowserInput;

    #[test]
    fn ime_commit_maps_to_browser_text_input() {
        let input = browser_keyboard_input(egui::Event::Ime(egui::ImeEvent::Commit(
            "\u{65e5}\u{672c}\u{8a9e}\u{5165}\u{529b}".to_owned(),
        )));

        assert_eq!(
            input,
            Some(HtmlBrowserInput::Text {
                text: "\u{65e5}\u{672c}\u{8a9e}\u{5165}\u{529b}".to_owned(),
            })
        );
    }

    #[test]
    fn ime_preedit_does_not_duplicate_committed_text() {
        let input = browser_keyboard_input(egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "\u{306b}\u{307b}\u{3093}\u{3054}".to_owned(),
            active_range_chars: Some(0..4),
        }));

        assert_eq!(input, None);
    }

    #[test]
    fn key_events_remain_key_events_and_never_become_text_input() {
        let down = browser_keyboard_input(egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        let up = browser_keyboard_input(egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        let text = browser_keyboard_input(egui::Event::Text("a".to_owned()));

        assert_eq!(
            down,
            Some(HtmlBrowserInput::KeyDown {
                key: "A".to_owned()
            })
        );
        assert_eq!(
            up,
            Some(HtmlBrowserInput::KeyUp {
                key: "A".to_owned()
            })
        );
        assert_eq!(
            text,
            Some(HtmlBrowserInput::Text {
                text: "a".to_owned()
            })
        );
    }
}
