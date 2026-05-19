use super::CheckUpdateError;
use serde::Deserialize;

pub(super) const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest";

#[derive(Debug)]
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
    pub(super) fn fetch_latest_release(
        url: &str,
    ) -> Result<Option<LatestRelease>, CheckUpdateError> {
        /* WHY: Try direct first. ureq 3.x's `Agent::new_with_defaults()` adopts
         * environment / system proxies automatically, which on some Windows
         * machines points at a stale or unreachable proxy and surfaces as a
         * "io: Connection refused" dialog at every launch even though
         * api.github.com is reachable directly. Direct-first sidesteps that
         * regression for users who do not need a proxy. */
        let direct_error = match Self::fetch_with_agent(url, &Self::direct_agent()) {
            Ok(release) => return Ok(Some(release)),
            Err(error) => CheckUpdateError::from(error),
        };

        /* If direct failed AND the user explicitly opted into a proxy via the
         * standard environment variables (corporate networks, intentional
         * traffic routing), try once more through it before giving up. */
        if ureq::Proxy::try_from_env().is_some() {
            match Self::fetch_with_agent(url, &ureq::Agent::new_with_defaults()) {
                Ok(release) => return Ok(Some(release)),
                Err(error) => return Err(CheckUpdateError::from(error)),
            }
        }

        Err(direct_error)
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
    use crate::update::proxy_env_test::ProxyEnvGuard;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn fetch_latest_release_succeeds_via_direct_even_when_env_proxy_is_refusing() {
        /* WHY: Regression guard for the v0.22.22+ Windows hang where ureq 3.x's default
         * agent picked up an env / system proxy that pointed at an unreachable address,
         * surfacing "io: Connection refused" to the user. Direct-first must succeed
         * for the common case (proxy env present but unreachable, GitHub reachable
         * directly) so the launch dialog stays clean. */
        let proxy_env = ProxyEnvGuard::capture();
        proxy_env.set_refusing_proxy();
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
    fn fetch_latest_release_maps_direct_connection_refusal() {
        let proxy_env = ProxyEnvGuard::capture();
        proxy_env.clear_proxy_env();
        let unreachable_url = format!("http://{}/latest", reserved_loopback_address());

        let error = ReleaseClient::fetch_latest_release(&unreachable_url).unwrap_err();

        assert_eq!(error, CheckUpdateError::NetworkUnreachable);
    }

    #[test]
    fn fetch_latest_release_maps_direct_and_env_proxy_failures() {
        let proxy_env = ProxyEnvGuard::capture();
        proxy_env.set_refusing_proxy();
        let unreachable_url = format!("http://{}/latest", reserved_loopback_address());

        let error = ReleaseClient::fetch_latest_release(&unreachable_url).unwrap_err();

        assert!(matches!(
            error,
            CheckUpdateError::NetworkUnreachable | CheckUpdateError::ProxyFailed
        ));
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
    fn fetch_latest_release_maps_non_2xx_status() {
        let proxy_env = ProxyEnvGuard::capture();
        proxy_env.clear_proxy_env();
        let url = spawn_release_server_with_status("HTTP/1.1 403 Forbidden", "{}");

        let error = ReleaseClient::fetch_latest_release(&url).unwrap_err();

        assert_eq!(error, CheckUpdateError::ServerStatus(403));
    }
}
