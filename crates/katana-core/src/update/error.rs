use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckUpdateError {
    NetworkUnreachable,
    NetworkTimedOut,
    ServerStatus(u16),
    ProxyFailed,
    InvalidPayload,
    StorageFull,
    PermissionDenied,
    InsufficientStorage {
        required_bytes: u64,
        available_bytes: u64,
    },
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
            Self::StorageFull => "update_install_error_storage_full",
            Self::PermissionDenied => "update_install_error_permission_denied",
            Self::InsufficientStorage { .. } => "update_install_error_insufficient_storage",
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
            Self::StorageFull => write!(formatter, "not enough disk space for update"),
            Self::PermissionDenied => write!(formatter, "update file access denied"),
            Self::InsufficientStorage {
                required_bytes,
                available_bytes,
            } => write!(
                formatter,
                "update needs {required_bytes} bytes; {available_bytes} bytes available"
            ),
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
    pub fn from_preparation_error(error: anyhow::Error) -> Self {
        if let Some(classified) = error.downcast_ref::<Self>() {
            return classified.clone();
        }
        for cause in error.chain() {
            if let Some(io) = cause.downcast_ref::<std::io::Error>() {
                match io.kind() {
                    std::io::ErrorKind::StorageFull => return Self::StorageFull,
                    std::io::ErrorKind::PermissionDenied => return Self::PermissionDenied,
                    _ => {}
                }
            }
        }
        match error.downcast::<ureq::Error>() {
            Ok(error) => Self::from(error),
            Err(error) => Self::Other(format!("{error:#}")),
        }
    }

    fn from_io_error(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::StorageFull => Self::StorageFull,
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied,
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
    fn preparation_retains_storage_error_through_context() {
        let error = anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::StorageFull))
            .context("extracting downloaded update");
        assert_eq!(
            CheckUpdateError::from_preparation_error(error),
            CheckUpdateError::StorageFull
        );
    }

    #[test]
    fn preparation_retains_permission_error_through_context() {
        let error = anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
            .context("creating update directory");
        assert_eq!(
            CheckUpdateError::from_preparation_error(error),
            CheckUpdateError::PermissionDenied
        );
    }

    #[test]
    fn preparation_retains_http_status_instead_of_unknown_error() {
        let error = anyhow::Error::new(ureq::Error::StatusCode(503));
        assert_eq!(
            CheckUpdateError::from_preparation_error(error),
            CheckUpdateError::ServerStatus(503)
        );
    }

    #[test]
    fn preparation_unknown_error_keeps_technical_chain() {
        let error = anyhow::anyhow!("invalid application bundle").context("preparing update");
        let mapped = CheckUpdateError::from_preparation_error(error);
        assert_eq!(
            mapped.technical_detail(),
            Some("preparing update: invalid application bundle")
        );
    }

    #[test]
    fn preparation_local_truncated_archive_is_not_a_network_failure() {
        let error = anyhow::Error::new(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "truncated local archive",
        ))
        .context("extracting downloaded update");
        let mapped = CheckUpdateError::from_preparation_error(error);
        assert_eq!(
            mapped.technical_detail(),
            Some("extracting downloaded update: truncated local archive")
        );
    }

    #[test]
    fn preparation_retains_required_and_available_storage_bytes() {
        let expected = CheckUpdateError::InsufficientStorage {
            required_bytes: 1700,
            available_bytes: 1300,
        };
        let error = anyhow::Error::new(expected.clone()).context("checking archive storage");
        assert_eq!(CheckUpdateError::from_preparation_error(error), expected);
    }

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
