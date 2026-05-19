mod ci_integrity;
mod windows_packaging;

use crate::Violation;
use std::path::Path;

pub struct ReleaseScriptOps;

impl ReleaseScriptOps {
    pub fn lint(root: &Path) -> Vec<Violation> {
        let mut violations = Self::lint_windows_msi_packaging(root);
        violations.extend(Self::lint_ci_integrity(root));
        violations
    }

    pub fn lint_windows_msi_packaging(root: &Path) -> Vec<Violation> {
        windows_packaging::WindowsPackagingOps::lint(root)
    }

    pub fn lint_ci_integrity(root: &Path) -> Vec<Violation> {
        ci_integrity::ReleaseCiIntegrityOps::lint(root)
    }
}
