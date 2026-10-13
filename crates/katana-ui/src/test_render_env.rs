use std::sync::{Mutex, MutexGuard};

static RENDER_ENV_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct RenderEnvLock;

impl RenderEnvLock {
    pub(crate) fn lock() -> MutexGuard<'static, ()> {
        RENDER_ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn with_lock<ResultValue>(action: impl FnOnce() -> ResultValue) -> ResultValue {
        let _guard = Self::lock();
        action()
    }
}
