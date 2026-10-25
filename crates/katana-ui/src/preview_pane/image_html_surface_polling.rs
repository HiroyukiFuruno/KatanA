use super::FRAME_UPDATE_POLL_INTERVAL;
use std::time::{Duration, Instant};

pub(super) fn next_poll_delay(
    deadline: &mut Option<Instant>,
    worker_busy: bool,
    now: Instant,
) -> Option<Duration> {
    if worker_busy {
        deadline.get_or_insert(now);
        Some(FRAME_UPDATE_POLL_INTERVAL)
    } else if let Some(limit) = *deadline {
        if now >= limit {
            *deadline = None;
        }
        Some(FRAME_UPDATE_POLL_INTERVAL)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_worker_keeps_polling_after_the_initial_window() {
        let now = Instant::now();
        let expired = now - FRAME_UPDATE_POLL_INTERVAL;
        let mut deadline = Some(expired);
        assert_eq!(
            next_poll_delay(&mut deadline, true, now),
            Some(FRAME_UPDATE_POLL_INTERVAL)
        );
        assert_eq!(deadline, Some(expired));
    }

    #[test]
    fn idle_transition_schedules_one_final_update_drain() {
        let now = Instant::now();
        let mut deadline = None;
        assert_eq!(
            next_poll_delay(&mut deadline, true, now),
            Some(FRAME_UPDATE_POLL_INTERVAL)
        );
        assert_eq!(
            next_poll_delay(&mut deadline, false, now),
            Some(FRAME_UPDATE_POLL_INTERVAL)
        );
        assert!(deadline.is_none());
        assert_eq!(next_poll_delay(&mut deadline, false, now), None);
    }

    #[test]
    fn pending_window_and_expired_window_stop_only_after_final_drain() {
        let now = Instant::now();
        let until = now + FRAME_UPDATE_POLL_INTERVAL;
        let mut deadline = Some(until);
        assert_eq!(
            next_poll_delay(&mut deadline, false, now),
            Some(FRAME_UPDATE_POLL_INTERVAL)
        );
        assert_eq!(deadline, Some(until));
        assert_eq!(
            next_poll_delay(&mut deadline, false, until),
            Some(FRAME_UPDATE_POLL_INTERVAL)
        );
        assert_eq!(next_poll_delay(&mut deadline, false, until), None);
    }
}
