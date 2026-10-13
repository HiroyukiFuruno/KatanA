use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckUpdateError {
    NetworkUnreachable,
    NetworkTimedOut,
    ServerStatus(u16),
    ProxyFailed,
    InvalidPayload,
    Other(String),
}

impl CheckUpdateError {
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::NetworkUnreachable => "update_check_error_network_unreachable",
            Self::NetworkTimedOut => "update_check_error_network_timed_out",
            Self::ServerStatus(_) => "update_check_error_server_status",
            Self::ProxyFailed => "update_check_error_proxy_failed",
            Self::InvalidPayload => "update_check_error_invalid_payload",
            Self::Other(_) => "update_check_error_unknown",
        }
    }

    pub fn technical_detail(&self) -> Option<&str> {
        match self {
            Self::Other(detail) => Some(detail),
            _ => None,
        }
    }
}

impl fmt::Display for CheckUpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkUnreachable => write!(formatter, "network unreachable"),
            Self::NetworkTimedOut => write!(formatter, "network timed out"),
            Self::ServerStatus(status) => write!(formatter, "server status: {status}"),
            Self::ProxyFailed => write!(formatter, "proxy failed"),
            Self::InvalidPayload => write!(formatter, "invalid update payload"),
            Self::Other(detail) => write!(formatter, "{detail}"),
        }
    }
}

impl std::error::Error for CheckUpdateError {}

impl From<ureq::Error> for CheckUpdateError {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::StatusCode(status) => Self::ServerStatus(status),
            ureq::Error::Timeout(_) => Self::NetworkTimedOut,
            ureq::Error::HostNotFound | ureq::Error::ConnectionFailed => Self::NetworkUnreachable,
            ureq::Error::InvalidProxyUrl | ureq::Error::ConnectProxyFailed(_) => Self::ProxyFailed,
            ureq::Error::Protocol(_)
            | ureq::Error::BodyStalled
            | ureq::Error::LargeResponseHeader(_, _)
            | ureq::Error::BodyExceedsLimit(_) => Self::InvalidPayload,
            ureq::Error::Io(error) => Self::from_io_error(error),
            ureq::Error::Json(_) => Self::InvalidPayload,
            _ => Self::Other(error.to_string()),
        }
    }
}

impl CheckUpdateError {
    fn from_io_error(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::TimedOut => Self::NetworkTimedOut,
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::AddrInUse
            | std::io::ErrorKind::AddrNotAvailable
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::UnexpectedEof => Self::NetworkUnreachable,
            _ => Self::Other(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_ureq_status_to_server_status() {
        assert_eq!(
            CheckUpdateError::from(ureq::Error::StatusCode(429)),
            CheckUpdateError::ServerStatus(429)
        );
    }

    #[test]
    fn maps_ureq_network_errors_to_network_variants() {
        assert_eq!(
            CheckUpdateError::from(ureq::Error::HostNotFound),
            CheckUpdateError::NetworkUnreachable
        );
        assert_eq!(
            CheckUpdateError::from(ureq::Error::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "timeout"
            ))),
            CheckUpdateError::NetworkTimedOut
        );
    }

    #[test]
    fn exposes_i18n_keys_for_all_variants() {
        assert_eq!(
            CheckUpdateError::NetworkUnreachable.i18n_key(),
            "update_check_error_network_unreachable"
        );
        assert_eq!(
            CheckUpdateError::NetworkTimedOut.i18n_key(),
            "update_check_error_network_timed_out"
        );
        assert_eq!(
            CheckUpdateError::ServerStatus(500).i18n_key(),
            "update_check_error_server_status"
        );
        assert_eq!(
            CheckUpdateError::ProxyFailed.i18n_key(),
            "update_check_error_proxy_failed"
        );
        assert_eq!(
            CheckUpdateError::InvalidPayload.i18n_key(),
            "update_check_error_invalid_payload"
        );
        assert_eq!(
            CheckUpdateError::Other("detail".to_string()).i18n_key(),
            "update_check_error_unknown"
        );
    }
}
