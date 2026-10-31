use crate::debug_log::DebugLog;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const MAX_EVENTS: usize = 512;
const SLOW_PHASE_US: u128 = 16_000;
static EVENTS: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct DocumentUiPerf;

pub(crate) struct DocumentUiSpan<'a> {
    phase: &'static str,
    path: Option<&'a Path>,
    start: Instant,
    paired: bool,
}

impl DocumentUiPerf {
    pub(crate) fn enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(DebugLog::is_enabled)
    }

    pub(crate) fn begin<'a>(
        phase: &'static str,
        path: Option<&'a Path>,
        paired: bool,
    ) -> Option<DocumentUiSpan<'a>> {
        if !Self::enabled() || (paired && !Self::reserve(2)) {
            return None;
        }
        let _ = DebugLog::monotonic_us();
        let span = DocumentUiSpan {
            phase,
            path,
            start: Instant::now(),
            paired,
        };
        if paired {
            span.emit("begin", 0);
        }
        Some(span)
    }

    fn reserve(count: usize) -> bool {
        /* WHY: 開始と終了を一緒に予約し、診断ログの上限で未終了と誤認する片側ログを作らない。 */
        EVENTS
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(count).filter(|next| *next <= MAX_EVENTS)
            })
            .is_ok()
    }
}

impl DocumentUiSpan<'_> {
    fn emit(&self, stage: &str, elapsed_us: u128) {
        DebugLog::write(
            "document_ui_perf",
            format_args!(
                "stage={stage} phase={} monotonic_us={} elapsed_us={elapsed_us} path={:?}",
                self.phase,
                DebugLog::monotonic_us(),
                self.path,
            ),
        );
    }
}

impl Drop for DocumentUiSpan<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros();
        if self.paired || (elapsed >= SLOW_PHASE_US && DocumentUiPerf::reserve(1)) {
            self.emit("end", elapsed);
        }
    }
}

#[cfg(test)]
#[path = "document_ui_perf_tests.rs"]
mod tests;
