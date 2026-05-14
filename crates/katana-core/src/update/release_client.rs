use anyhow::Result;
use serde::Deserialize;
use std::io::ErrorKind;

pub(super) const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest";

pub(super) struct LatestRelease {
    pub(super) tag_name: String,
    pub(super) html_url: String,
    pub(super) body: String,
}

pub(super) struct ReleaseClient;

impl ReleaseClient {
    pub(super) fn fetch_latest_release(url: &str) -> Result<LatestRelease> {
        match Self::fetch_with_agent(url, &ureq::Agent::new_with_defaults()) {
            Ok(release) => Ok(release),
            Err(error) if Self::should_retry_without_proxy(&error) => {
                Self::fetch_with_agent(url, &Self::direct_agent()).map_err(Into::into)
            }
            Err(error) => Err(error.into()),
        }
    }

    fn direct_agent() -> ureq::Agent {
        let config = ureq::Agent::config_builder().proxy(None).build();
        ureq::Agent::new_with_config(config)
    }

    fn fetch_with_agent(url: &str, agent: &ureq::Agent) -> Result<LatestRelease, ureq::Error> {
        let payload = agent
            .get(url)
            .header("User-Agent", "KatanA-Update-Manager")
            .header("Accept", "application/vnd.github+json")
            .call()?
            .body_mut()
            .read_json::<GitHubReleasePayload>()?;
        Ok(payload.into())
    }

    fn should_retry_without_proxy(error: &ureq::Error) -> bool {
        if ureq::Proxy::try_from_env().is_none() {
            return false;
        }

        match error {
            ureq::Error::Io(error) => error.kind() == ErrorKind::ConnectionRefused,
            ureq::Error::ConnectProxyFailed(message) => {
                message.to_ascii_lowercase().contains("refused")
            }
            _ => false,
        }
    }
}

#[derive(Deserialize)]
struct GitHubReleasePayload {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: String,
}

impl From<GitHubReleasePayload> for LatestRelease {
    fn from(value: GitHubReleasePayload) -> Self {
        Self {
            tag_name: value.tag_name,
            html_url: value.html_url,
            body: value.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Mutex;

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

    struct EnvSnapshot {
        values: Vec<(&'static str, Option<String>)>,
    }

    impl EnvSnapshot {
        fn capture() -> Self {
            let values = PROXY_ENV_KEYS
                .iter()
                .map(|key| (*key, std::env::var(key).ok()))
                .collect();
            Self { values }
        }

        fn set_refusing_proxy() {
            for key in [
                "all_proxy",
                "ALL_PROXY",
                "https_proxy",
                "HTTP_PROXY",
                "http_proxy",
            ] {
                /* WHY: Environment variables are process-wide, so this test mutates them only
                while holding PROXY_ENV_MUTEX. */
                unsafe { std::env::set_var(key, "http://127.0.0.1:1") };
            }
            for key in ["NO_PROXY", "no_proxy"] {
                /* WHY: localhost proxy bypass would hide the refused-proxy regression. */
                unsafe { std::env::remove_var(key) };
            }
        }
    }

    impl Drop for EnvSnapshot {
        fn drop(&mut self) {
            for (key, value) in &self.values {
                match value {
                    Some(value) => unsafe { std::env::set_var(key, value) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
    }

    #[test]
    fn fetch_latest_release_retries_direct_when_env_proxy_refuses_connection() {
        let _guard = PROXY_ENV_MUTEX.lock().unwrap();
        let _snapshot = EnvSnapshot::capture();
        EnvSnapshot::set_refusing_proxy();
        let url = spawn_release_server(
            r#"{"tag_name":"v9.9.9","html_url":"https://example.test/release","body":"notes"}"#,
        );

        let release = ReleaseClient::fetch_latest_release(&url).unwrap();

        assert_eq!(release.tag_name, "v9.9.9");
        assert_eq!(release.html_url, "https://example.test/release");
        assert_eq!(release.body, "notes");
    }

    fn spawn_release_server(payload: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                payload
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        format!("http://{address}/latest")
    }
}
