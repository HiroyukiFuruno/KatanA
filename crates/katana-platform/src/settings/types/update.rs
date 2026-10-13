use serde::{Deserialize, Serialize};
use std::time::Duration;

const UPDATE_INTERVAL_5_MIN: u64 = 5;
const UPDATE_INTERVAL_15_MIN: u64 = 15;
const UPDATE_INTERVAL_30_MIN: u64 = 30;
const UPDATE_INTERVAL_1_HOUR: u64 = 1;
const UPDATE_INTERVAL_3_HOURS: u64 = 3;
const UPDATE_INTERVAL_6_HOURS: u64 = 6;
const UPDATE_INTERVAL_8_HOURS: u64 = 8;
const UPDATE_INTERVAL_12_HOURS: u64 = 12;
const UPDATE_INTERVAL_24_HOURS: u64 = 24;
const UPDATE_INTERVAL_7_DAYS: u64 = 7;
const UPDATE_INTERVAL_30_DAYS: u64 = 30;
const SECONDS_PER_MINUTE: u64 = 60;
const UPDATE_INTERVAL_60_MINUTES: u64 = 60;
const UI_INTERVAL_OPTION_COUNT: usize = 10;

/* WHY: Interval for checking for application updates. */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UpdateInterval {
    /* WHY: Skip automatic updates */
    #[serde(rename = "Never")]
    Never,
    /* WHY: Check for updates every 5 minutes. */
    #[serde(rename = "5min")]
    FiveMinutes,
    /* WHY: Check for updates every 15 minutes. */
    #[serde(rename = "15min")]
    FifteenMinutes,
    /* WHY: Check for updates every 30 minutes. */
    #[serde(rename = "30min")]
    ThirtyMinutes,
    /* WHY: Check for updates every hour. */
    #[serde(rename = "1h")]
    OneHour,
    /* WHY: Check for updates every 3 hours. */
    #[serde(rename = "3h")]
    ThreeHours,
    /* WHY: Check for updates every 6 hours. */
    #[serde(rename = "6h")]
    SixHours,
    /* WHY: Check for updates every 8 hours. */
    #[serde(rename = "8h")]
    EightHours,
    /* WHY: Check for updates every 12 hours. */
    #[serde(rename = "12h")]
    TwelveHours,
    /* WHY: Check for updates daily */
    #[default]
    #[serde(rename = "Daily")]
    Daily,
    /* WHY: Check for updates weekly */
    #[serde(rename = "Weekly")]
    Weekly,
    /* WHY: Check for updates monthly */
    #[serde(rename = "Monthly")]
    Monthly,
}

impl UpdateInterval {
    pub const fn ui_options() -> &'static [Self; UI_INTERVAL_OPTION_COUNT] {
        &[
            Self::Never,
            Self::FiveMinutes,
            Self::FifteenMinutes,
            Self::ThirtyMinutes,
            Self::OneHour,
            Self::ThreeHours,
            Self::SixHours,
            Self::EightHours,
            Self::TwelveHours,
            Self::Daily,
        ]
    }

    pub const fn as_duration(self) -> Option<Duration> {
        match self {
            Self::Never => None,
            Self::FiveMinutes => Some(Duration::from_secs(
                UPDATE_INTERVAL_5_MIN * SECONDS_PER_MINUTE,
            )),
            Self::FifteenMinutes => Some(Duration::from_secs(
                UPDATE_INTERVAL_15_MIN * SECONDS_PER_MINUTE,
            )),
            Self::ThirtyMinutes => Some(Duration::from_secs(
                UPDATE_INTERVAL_30_MIN * SECONDS_PER_MINUTE,
            )),
            Self::OneHour => Some(Duration::from_secs(
                UPDATE_INTERVAL_1_HOUR * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::ThreeHours => Some(Duration::from_secs(
                UPDATE_INTERVAL_3_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::SixHours => Some(Duration::from_secs(
                UPDATE_INTERVAL_6_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::EightHours => Some(Duration::from_secs(
                UPDATE_INTERVAL_8_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::TwelveHours => Some(Duration::from_secs(
                UPDATE_INTERVAL_12_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::Daily => Some(Duration::from_secs(
                UPDATE_INTERVAL_24_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::Weekly => Some(Duration::from_secs(
                UPDATE_INTERVAL_7_DAYS * UPDATE_INTERVAL_24_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
            Self::Monthly => Some(Duration::from_secs(
                UPDATE_INTERVAL_30_DAYS * UPDATE_INTERVAL_24_HOURS * UPDATE_INTERVAL_60_MINUTES,
            )),
        }
    }

    pub const fn as_pulldown_label(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::FiveMinutes => "5min",
            Self::FifteenMinutes => "15min",
            Self::ThirtyMinutes => "30min",
            Self::OneHour => "1h",
            Self::ThreeHours => "3h",
            Self::SixHours => "6h",
            Self::EightHours => "8h",
            Self::TwelveHours => "12h",
            Self::Daily => "24h",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
        }
    }
}

/* WHY: Auto-updater configuration. */
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSettings {
    /* WHY: The interval at which the app should check for updates. */
    #[serde(default)]
    pub interval: UpdateInterval,
    /* WHY: The last time an update check was performed (UNIX timestamp in seconds). */
    #[serde(default)]
    pub last_checked_timestamp_sec: Option<u64>,
    /* WHY: Version tag the user explicitly chose to skip (e.g. "v0.8.0").
    Auto-check will suppress notifications for this version. */
    #[serde(default)]
    pub skipped_version: Option<String>,
    /* WHY: The application version recorded during the previous launch.
    Used to determine whether to show the release notes after an update. */
    #[serde(default)]
    pub previous_app_version: Option<String>,
}
