use super::CheckUpdateError;
use serde::Deserialize;
use ureq::ResponseExt;

pub(super) const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest";
const LATEST_RELEASE_PAGE_URL: &str = "https://github.com/HiroyukiFuruno/KatanA/releases/latest";

#[derive(Debug)]
pub(super) struct LatestRelease {
    pub(super) tag_name: String,
    pub(super) html_url: String,
    pub(super) body: String,
}

pub(super) struct ReleaseClient;

impl ReleaseClient {
    pub(super) fn fetch_latest_release(
        url: &str,
    ) -> Result<Option<LatestRelease>, CheckUpdateError> {
        let page_url = (url == LATEST_RELEASE_API_URL).then_some(LATEST_RELEASE_PAGE_URL);
        Self::fetch_latest_release_from_sources(url, page_url)
    }

    fn fetch_latest_release_from_sources(
        api_url: &str,
        page_url: Option<&str>,
    ) -> Result<Option<LatestRelease>, CheckUpdateError> {
        let api_error = match Self::fetch_with_agents(api_url, Self::fetch_api_with_agent) {
            Ok(release) => return Ok(Some(release)),
            Err(error) => error,
        };
        let Some(page_url) = page_url.filter(|_| Self::is_api_rate_limited(&api_error)) else {
            return Err(api_error);
        };
        Self::fetch_with_agents(page_url, Self::fetch_page_with_agent).map(Some)
    }

    fn is_api_rate_limited(error: &CheckUpdateError) -> bool {
        matches!(error, CheckUpdateError::ServerStatus(403 | 429))
    }

    fn fetch_with_agents(
        url: &str,
        fetch: fn(&str, &ureq::Agent) -> Result<LatestRelease, CheckUpdateError>,
    ) -> Result<LatestRelease, CheckUpdateError> {
        let direct_error = match fetch(url, &Self::direct_agent()) {
            Ok(release) => return Ok(release),
            Err(error) => error,
        };

        if !Self::should_retry_via_proxy(&direct_error) || ureq::Proxy::try_from_env().is_none() {
            return Err(direct_error);
        }
        fetch(url, &ureq::Agent::new_with_defaults())
    }

    fn should_retry_via_proxy(error: &CheckUpdateError) -> bool {
        matches!(
            error,
            CheckUpdateError::NetworkUnreachable
                | CheckUpdateError::NetworkTimedOut
                | CheckUpdateError::ProxyFailed
        )
    }

    fn direct_agent() -> ureq::Agent {
        let config = ureq::Agent::config_builder().proxy(None).build();
        ureq::Agent::new_with_config(config)
    }

    fn fetch_api_with_agent(
        url: &str,
        agent: &ureq::Agent,
    ) -> Result<LatestRelease, CheckUpdateError> {
        let payload = agent
            .get(url)
            .header("User-Agent", "KatanA-Update-Manager")
            .header("Accept", "application/vnd.github+json")
            .call()?
            .body_mut()
            .read_json::<GitHubReleasePayload>()?;
        Ok(payload.into())
    }

    fn fetch_page_with_agent(
        url: &str,
        agent: &ureq::Agent,
    ) -> Result<LatestRelease, CheckUpdateError> {
        let response = agent
            .get(url)
            .header("User-Agent", "KatanA-Update-Manager")
            .call()?;
        let html_url = response.get_uri().to_string();
        let tag_name =
            Self::release_tag_from_url(&html_url).ok_or(CheckUpdateError::InvalidPayload)?;
        Ok(LatestRelease {
            tag_name,
            html_url,
            body: String::new(),
        })
    }

    fn release_tag_from_url(url: &str) -> Option<String> {
        let (_, suffix) = url.split_once("/releases/tag/")?;
        let tag = suffix.split(['?', '#']).next()?;
        if tag.is_empty() || tag.contains('/') {
            return None;
        }
        semver::Version::parse(tag.strip_prefix('v').unwrap_or(tag)).ok()?;
        Some(tag.to_owned())
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

    #[test]
    fn fetch_latest_release_recovers_from_rate_limit_via_public_release_redirect() {
        let proxy_env = ProxyEnvGuard::capture();
        proxy_env.set_refusing_proxy();
        for status in ["403 Forbidden", "429 Too Many Requests"] {
            let (api_url, page_url) = spawn_rate_limited_release_server(status);

            let release =
                ReleaseClient::fetch_latest_release_from_sources(&api_url, Some(page_url.as_str()))
                    .unwrap()
                    .expect("release redirect returned");

            assert_eq!(release.tag_name, "v9.9.9");
            assert_eq!(
                release.html_url,
                format!("{}/releases/tag/v9.9.9", server_origin(&api_url))
            );
            assert!(release.body.is_empty());
        }
    }

    #[test]
    fn recognizes_only_rate_limit_statuses_for_release_page_fallback() {
        assert!(ReleaseClient::is_api_rate_limited(
            &CheckUpdateError::ServerStatus(403)
        ));
        assert!(ReleaseClient::is_api_rate_limited(
            &CheckUpdateError::ServerStatus(429)
        ));
        assert!(!ReleaseClient::is_api_rate_limited(
            &CheckUpdateError::ServerStatus(500)
        ));
    }

    #[test]
    fn release_redirect_requires_a_semver_tag_without_nested_path() {
        assert_eq!(
            ReleaseClient::release_tag_from_url(
                "https://github.com/HiroyukiFuruno/KatanA/releases/tag/v0.22.35"
            ),
            Some("v0.22.35".to_owned())
        );
        assert_eq!(
            ReleaseClient::release_tag_from_url("https://example.test/releases/tag/not-semver"),
            None
        );
        assert_eq!(
            ReleaseClient::release_tag_from_url("https://example.test/releases/tag/v1.2.3/file"),
            None
        );
        assert_eq!(
            ReleaseClient::release_tag_from_url("https://example.test/releases/latest"),
            None
        );
    }

    fn spawn_rate_limited_release_server(status: &'static str) -> (String, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let redirect_origin = origin.clone();
        std::thread::spawn(move || {
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 1024];
                let size = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..size]);
                let path = request.split_whitespace().nth(1).unwrap();
                let response = rate_limited_server_response(path, &redirect_origin, status);
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (
            format!("{origin}/api/latest"),
            format!("{origin}/releases/latest"),
        )
    }

    fn rate_limited_server_response(path: &str, origin: &str, status: &str) -> String {
        match path {
            "/api/latest" => http_response(status, "{}", None),
            "/releases/latest" => http_response(
                "302 Found",
                "",
                Some(&format!("{origin}/releases/tag/v9.9.9")),
            ),
            "/releases/tag/v9.9.9" => http_response("200 OK", "release", None),
            _ => http_response("404 Not Found", "not found", None),
        }
    }

    fn http_response(status: &str, body: &str, location: Option<&str>) -> String {
        let location = location
            .map(|url| format!("Location: {url}\r\n"))
            .unwrap_or_default();
        format!(
            "HTTP/1.1 {status}\r\n{location}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn server_origin(url: &str) -> &str {
        url.strip_suffix("/api/latest").unwrap()
    }
}
