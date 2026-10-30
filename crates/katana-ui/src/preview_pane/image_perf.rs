use std::collections::VecDeque;
use std::fmt::Arguments;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct ImageRequestTrace {
    pub(super) owner: usize,
    pub(super) generation: u64,
    pub(super) revision: u64,
}

pub(crate) struct ImagePerf;
type SeenTransition = (
    ImageRequestTrace,
    PathBuf,
    Option<eframe::egui::TextureId>,
    &'static str,
);
static TRANSITIONS: OnceLock<Mutex<VecDeque<SeenTransition>>> = OnceLock::new();

impl ImagePerf {
    pub(crate) fn enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(crate::debug_log::DebugLog::is_enabled)
    }

    pub(super) fn request(
        owner: usize,
        generation: u64,
        revision: u64,
    ) -> Option<ImageRequestTrace> {
        Self::enabled().then_some(ImageRequestTrace {
            owner,
            generation,
            revision,
        })
    }

    pub(crate) fn selection(path: &Path, activate: bool) {
        if Self::enabled() && katana_core::workspace::TreeEntry::path_is_image(path) {
            if activate
                && let Some(seen) = TRANSITIONS.get()
                && let Ok(mut seen) = seen.lock()
            {
                seen.retain(|(_, candidate, _, _)| candidate != path);
            }
            Self::write(
                "selection",
                path,
                format_args!(
                    "owner=unassigned generation=unassigned revision=unassigned activate={activate}"
                ),
            );
        }
    }

    pub(super) fn emit(
        trace: Option<ImageRequestTrace>,
        stage: &'static str,
        path: &Path,
        details: Arguments<'_>,
    ) {
        if let Some(trace) = trace {
            Self::write(
                stage,
                path,
                format_args!(
                    "owner={:x} generation={} revision={} {details}",
                    trace.owner, trace.generation, trace.revision
                ),
            );
        }
    }

    pub(super) fn once(
        trace: Option<ImageRequestTrace>,
        stage: &'static str,
        path: &Path,
        texture: Option<eframe::egui::TextureId>,
    ) {
        let Some(trace) = trace else {
            return;
        };
        let Ok(mut seen) = TRANSITIONS.get_or_init(Mutex::default).lock() else {
            return;
        };
        if seen.iter().any(|(identity, candidate, id, event)| {
            *identity == trace && candidate == path && *id == texture && *event == stage
        }) {
            return;
        }
        /* WHY: DEBUG専用の重複抑制も上限を設け、長時間の診断で保持し続けない。 */
        if seen.len() == 512 {
            seen.pop_front();
        }
        seen.push_back((trace, path.to_path_buf(), texture, stage));
        drop(seen);
        Self::emit(
            Some(trace),
            stage,
            path,
            format_args!("texture={texture:?} gpu_complete=false"),
        );
    }

    fn write(stage: &str, path: &Path, details: Arguments<'_>) {
        crate::debug_log::DebugLog::write(
            "image_perf",
            format_args!(
                "stage={stage} monotonic_us={} path={path:?} {details}",
                crate::debug_log::DebugLog::monotonic_us()
            ),
        );
    }
}

#[cfg(test)]
#[path = "image_perf_tests.rs"]
mod tests;
