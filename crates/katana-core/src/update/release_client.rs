use anyhow::Result;
use serde::Deserialize;

pub(super) const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest";

pub(super) struct LatestRelease {
    pub(super) tag_name: String,
    pub(super) html_url: String,
    pub(super) body: String,
}

pub(super) struct ReleaseClient;

impl ReleaseClient {
    /* WHY: `error-first` ast-lint rule forbids `if let Ok(...) {}` success-branching,
     * `clippy::single_match` complains that our two-arm match collapses to `if let`.
     * The two rules are mutually exclusive here. The match form makes the
     * "Ok → early return, Err → fall through to next strategy" intent the most
     * legible, so silence the clippy lint locally rather than the project-wide rule. */
    #[allow(clippy::single_match)]
    pub(super) fn fetch_latest_release(url: &str) -> Result<Option<LatestRelease>> {
        /* WHY: Try direct first. ureq 3.x's `Agent::new_with_defaults()` adopts
         * environment / system proxies automatically, which on some Windows
         * machines points at a stale or unreachable proxy and surfaces as a
         * "io: Connection refused" dialog at every launch even though
         * api.github.com is reachable directly. Direct-first sidesteps that
         * regression for users who do not need a proxy. */
        match Self::fetch_with_agent(url, &Self::direct_agent()) {
            Ok(release) => return Ok(Some(release)),
            Err(_) => {}
        }

        /* If direct failed AND the user explicitly opted into a proxy via the
         * standard environment variables (corporate networks, intentional
         * traffic routing), try once more through it before giving up. */
        if ureq::Proxy::try_from_env().is_some() {
            match Self::fetch_with_agent(url, &ureq::Agent::new_with_defaults()) {
                Ok(release) => return Ok(Some(release)),
                Err(_) => {}
            }
        }

        /* WHY: Update check is a best-effort background convenience, not a
         * critical path. Any unresolved fetch (network refused, DNS failure,
         * rate-limit 403/429, malformed payload) is collapsed to "no update
         * info available right now" so the launch UI never has to show a red
         * update-check-failed dialog. Users can still browse the GitHub
         * Releases page manually. */
        Ok(None)
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

        fn clear_proxy_env() {
            for key in PROXY_ENV_KEYS {
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
    fn fetch_latest_release_succeeds_via_direct_even_when_env_proxy_is_refusing() {
        /* WHY: Regression guard for the v0.22.22+ Windows hang where ureq 3.x's default
         * agent picked up an env / system proxy that pointed at an unreachable address,
         * surfacing "io: Connection refused" to the user. Direct-first must succeed
         * for the common case (proxy env present but unreachable, GitHub reachable
         * directly) so the launch dialog stays clean. */
        let _guard = PROXY_ENV_MUTEX.lock().unwrap();
        let _snapshot = EnvSnapshot::capture();
        EnvSnapshot::set_refusing_proxy();
        let url = spawn_release_server(
            r#"{"tag_name":"v9.9.9","html_url":"https://example.test/release","body":"notes"}"#,
        );

        let release = ReleaseClient::fetch_latest_release(&url)
            .unwrap()
            .expect("release payload returned");

        assert_eq!(release.tag_name, "v9.9.9");
        assert_eq!(release.html_url, "https://example.test/release");
        assert_eq!(release.body, "notes");
    }

    #[test]
    fn fetch_latest_release_returns_none_when_direct_connection_is_refused() {
        /* WHY: When the server is unreachable AND no env proxy is configured,
         * the update check must collapse to Ok(None) — never bubble a network
         * error to the launch UI as a red update-check-failed dialog. */
        let _guard = PROXY_ENV_MUTEX.lock().unwrap();
        let _snapshot = EnvSnapshot::capture();
        EnvSnapshot::clear_proxy_env();
        let unreachable_url = format!("http://{}/latest", reserved_loopback_address());

        let result = ReleaseClient::fetch_latest_release(&unreachable_url).unwrap();

        assert!(
            result.is_none(),
            "network refusal must not propagate as Err"
        );
    }

    #[test]
    fn fetch_latest_release_returns_none_when_direct_and_env_proxy_both_fail() {
        /* WHY: Even if the user has an env proxy configured (corporate network)
         * AND both direct and proxy paths fail, the update check must still
         * collapse to Ok(None) rather than surface a launch-blocking dialog. */
        let _guard = PROXY_ENV_MUTEX.lock().unwrap();
        let _snapshot = EnvSnapshot::capture();
        EnvSnapshot::set_refusing_proxy();
        let unreachable_url = format!("http://{}/latest", reserved_loopback_address());

        let result = ReleaseClient::fetch_latest_release(&unreachable_url).unwrap();

        assert!(
            result.is_none(),
            "both direct and proxy failures must collapse to Ok(None)"
        );
    }

    fn reserved_loopback_address() -> String {
        /* WHY: Bind a TCP listener to grab a free port, then immediately drop it so
         * subsequent connects deterministically receive RST → ConnectionRefused.
         * `127.0.0.1:1` is sometimes occupied by a local service or filtered by the OS,
         * so picking a verified-unused port avoids platform-specific test flakes. */
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        address.to_string()
    }

    fn spawn_release_server(payload: &'static str) -> String {
        spawn_release_server_with_status("HTTP/1.1 200 OK", payload)
    }

    fn spawn_release_server_with_status(
        status_line: &'static str,
        payload: &'static str,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                payload
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        format!("http://{address}/latest")
    }

    #[test]
    fn fetch_latest_release_returns_none_on_non_2xx_status() {
        let _guard = PROXY_ENV_MUTEX.lock().unwrap();
        let _snapshot = EnvSnapshot::capture();
        EnvSnapshot::clear_proxy_env();
        let url = spawn_release_server_with_status("HTTP/1.1 403 Forbidden", "{}");

        let result = ReleaseClient::fetch_latest_release(&url).unwrap();

        assert!(result.is_none());
    }
}
