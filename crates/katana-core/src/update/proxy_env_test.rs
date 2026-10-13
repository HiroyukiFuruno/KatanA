use std::sync::{Mutex, MutexGuard};

static PROXY_ENV_MUTEX: Mutex<()> = Mutex::new(());

const PROXY_ENV_KEYS: &[&str] = &[
    "all_proxy",
    "ALL_PROXY",
    "https_proxy",
    "HTTP_PROXY",
    "http_proxy",
    "NO_PROXY",
    "no_proxy",
];

pub(super) struct ProxyEnvGuard {
    _lock: MutexGuard<'static, ()>,
    values: Vec<(&'static str, Option<String>)>,
}

impl ProxyEnvGuard {
    pub(super) fn capture() -> Self {
        let lock = PROXY_ENV_MUTEX.lock().expect("Test requirement");
        let values = PROXY_ENV_KEYS
            .iter()
            .map(|key| (*key, std::env::var(key).ok()))
            .collect();
        Self {
            _lock: lock,
            values,
        }
    }

    pub(super) fn set_refusing_proxy(&self) {
        for key in [
            "all_proxy",
            "ALL_PROXY",
            "https_proxy",
            "HTTP_PROXY",
            "http_proxy",
        ] {
            unsafe { std::env::set_var(key, "http://127.0.0.1:1") };
        }
        for key in ["NO_PROXY", "no_proxy"] {
            unsafe { std::env::remove_var(key) };
        }
    }

    pub(super) fn clear_proxy_env(&self) {
        for key in PROXY_ENV_KEYS {
            unsafe { std::env::remove_var(key) };
        }
    }
}

impl Drop for ProxyEnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.values {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}
