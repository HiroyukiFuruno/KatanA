use super::types::*;
use eframe::egui;

use super::types::PreviewPane;

fn apply_section_msg(
    sections: &mut Vec<RenderedSection>,
    kind: &str,
    source: &str,
    section: RenderedSection,
) {
    for slot in sections {
        let RenderedSection::Pending {
            kind: p_kind,
            source: p_source,
            ..
        } = slot
        else {
            continue;
        };
        if p_kind == kind && p_source == source {
            *slot = section.clone();
            return;
        }
    }
}

impl PreviewPane {
    fn finalize_disconnected_renders(&mut self) {
        for slot in &mut self.sections {
            if let RenderedSection::Pending {
                kind,
                source,
                source_lines,
            } = slot.clone()
            {
                *slot = RenderedSection::Error {
                    kind,
                    _source: source,
                    message: "Diagram render worker disconnected before producing a result."
                        .to_string(),
                    source_lines,
                };
            }
        }

        self.is_loading = false;
        self.render_rx = None;
    }

    pub(crate) fn poll_renders(&mut self, ctx: &egui::Context) {
        while let Some(path) = self.image_preload_queue.pop() {
            if self.image_cache.insert(path.clone()) {
                let uri = format!("file://{}", path.display());
                let _ = ctx.try_load_image(&uri, egui::load::SizeHint::Scale(1.0.into()));
            }
        }

        let mut disconnected = false;

        if let Some(rx) = &self.render_rx {
            loop {
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        RenderMessage::Section {
                            kind,
                            source,
                            section,
                        } => {
                            apply_section_msg(&mut self.sections, &kind, &source, section);
                        }
                        RenderMessage::ReduceConcurrency => {
                            self.concurrency_reduction_requested = true;
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }

            if disconnected {
                self.finalize_disconnected_renders();
            }
        } else {
            self.is_loading = false;
        }
    }

    pub fn wait_for_renders(&mut self) {
        while let Some(rx) = &self.render_rx {
            let mut disconnected = false;

            loop {
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        RenderMessage::Section {
                            kind,
                            source,
                            section,
                        } => {
                            apply_section_msg(&mut self.sections, &kind, &source, section);
                        }
                        RenderMessage::ReduceConcurrency => {
                            self.concurrency_reduction_requested = true;
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }

            if disconnected {
                self.finalize_disconnected_renders();
                break;
            }

            if self
                .sections
                .iter()
                .any(|s| matches!(s, RenderedSection::Pending { .. }))
            {
                std::thread::sleep(std::time::Duration::from_millis(RENDER_POLL_INTERVAL_MS));
            } else {
                self.render_rx = None;
                break;
            }
        }
    }
}
