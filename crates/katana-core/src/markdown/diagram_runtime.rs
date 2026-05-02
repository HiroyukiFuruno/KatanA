#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiagramRuntimeMode;

impl DiagramRuntimeMode {
    pub(crate) fn current() -> Self {
        Self
    }

    pub(crate) fn mermaid_cache_profile(self) -> &'static str {
        "rust-managed-js-svg"
    }

    pub(crate) fn mermaid_cache_extension(self) -> &'static str {
        "svg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mermaid_runtime_is_always_rust_managed_svg() {
        let mode = DiagramRuntimeMode::current();
        assert_eq!(mode.mermaid_cache_profile(), "rust-managed-js-svg");
        assert_eq!(mode.mermaid_cache_extension(), "svg");
    }
}
