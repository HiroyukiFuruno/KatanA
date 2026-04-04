#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rustc_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutInfo {
    pub product_name: &'static str,
    pub version: &'static str,
    pub build: &'static str,
    pub copyright: &'static str,
    pub license: &'static str,
    pub description: &'static str,
    pub repository: &'static str,
    pub docs_url: &'static str,
    pub issues_url: &'static str,
    pub sponsor_url: &'static str,
    pub system: SystemInfo,
}

pub struct AboutInfoOps;
