use super::DocumentWorkerLifecycle;
use crate::test_render_env::RenderEnvLock;
use std::sync::MutexGuard;
use std::time::{Duration, Instant};

const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) struct HtmlRuntimeTestGuard {
    _environment: MutexGuard<'static, ()>,
}

impl HtmlRuntimeTestGuard {
    pub(crate) fn lock() -> Self {
        Self {
            _environment: RenderEnvLock::lock(),
        }
    }
}

impl Drop for HtmlRuntimeTestGuard {
    fn drop(&mut self) {
        /* WHY: 非同期closeが終わる前に次のテストへruntime環境を明け渡さない。 */
        let deadline = Instant::now() + CLOSE_TIMEOUT;
        while DocumentWorkerLifecycle::live_count() != 0 && Instant::now() < deadline {
            std::thread::yield_now();
        }
        if !std::thread::panicking() {
            assert_eq!(
                DocumentWorkerLifecycle::live_count(),
                0,
                "HTML cleanup exceeded the existing five-second close budget"
            );
        }
    }
}
