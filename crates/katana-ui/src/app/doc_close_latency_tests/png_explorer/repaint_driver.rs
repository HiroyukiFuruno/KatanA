use eframe::egui::{self, RequestRepaintInfo};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Instant;

pub(super) struct RepaintDriver {
    receiver: Receiver<(Instant, RequestRepaintInfo)>,
    next_due: Option<Instant>,
    pub(super) callbacks: usize,
}

impl RepaintDriver {
    pub(super) fn install(context: &egui::Context) -> Self {
        let (sender, receiver) = mpsc::channel();
        context.set_request_repaint_callback(move |info| {
            let _ = sender.send((Instant::now(), info));
        });
        Self {
            receiver,
            next_due: None,
            callbacks: 0,
        }
    }

    pub(super) fn record_output(&mut self, output: &egui::FullOutput, started: Instant) {
        let delay = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .expect("root viewport output")
            .repaint_delay;
        self.next_due = Instant::now().checked_add(delay);
        eprintln!(
            "png_explorer_on_demand_output elapsed_ms={} delay_ms={}",
            started.elapsed().as_secs_f64() * 1000.0,
            delay.as_secs_f64() * 1000.0
        );
    }

    pub(super) fn wait_until_due(
        &mut self,
        context: &egui::Context,
        started: Instant,
        deadline: Instant,
    ) {
        loop {
            while let Ok((requested_at, info)) = self.receiver.try_recv() {
                self.accept_callback(context, started, requested_at, info);
            }
            let now = Instant::now();
            assert!(
                now < deadline,
                "PNG on-demand repaint deadline exceeded: elapsed_ms={} callbacks={} next_due={:?}",
                started.elapsed().as_secs_f64() * 1000.0,
                self.callbacks,
                self.next_due
            );
            if self.next_due.is_some_and(|due| due <= now) {
                self.next_due = None;
                return;
            }
            let wake_at = self.next_due.map_or(deadline, |due| due.min(deadline));
            match self
                .receiver
                .recv_timeout(wake_at.saturating_duration_since(now))
            {
                Ok((requested_at, info)) => {
                    self.accept_callback(context, started, requested_at, info)
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("repaint callback channel disconnected")
                }
            }
        }
    }

    fn accept_callback(
        &mut self,
        context: &egui::Context,
        started: Instant,
        requested_at: Instant,
        info: RequestRepaintInfo,
    ) {
        self.callbacks += 1;
        let current_pass = context.cumulative_pass_nr();
        /* WHY: native eframeと同じく、既に二pass以上進んだ古い再描画要求は再実行しない。 */
        let accepted = info.viewport_id == egui::ViewportId::ROOT
            && (current_pass == info.current_cumulative_pass_nr
                || current_pass == info.current_cumulative_pass_nr.saturating_add(1));
        eprintln!(
            "png_explorer_on_demand_callback elapsed_ms={} delay_ms={} requested_pass={} current_pass={} accepted={accepted}",
            requested_at
                .saturating_duration_since(started)
                .as_secs_f64()
                * 1000.0,
            info.delay.as_secs_f64() * 1000.0,
            info.current_cumulative_pass_nr,
            current_pass
        );
        if accepted && let Some(due) = requested_at.checked_add(info.delay) {
            self.next_due = Some(self.next_due.map_or(due, |previous| previous.min(due)));
        }
    }
}
