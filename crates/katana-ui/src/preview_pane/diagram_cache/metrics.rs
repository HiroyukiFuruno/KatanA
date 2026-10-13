use katana_core::markdown::DiagramKind;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DiagramCacheMetric {
    Hit,
    Miss,
    Pruned,
    CorruptSvg,
    RedrawExecuted,
    SkippedByTabSwitch,
}

impl DiagramCacheMetric {
    fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "diagram_cache_hit",
            Self::Miss => "diagram_cache_miss",
            Self::Pruned => "diagram_cache_pruned",
            Self::CorruptSvg => "diagram_cache_corrupt_svg",
            Self::RedrawExecuted => "diagram_cache_redraw_executed",
            Self::SkippedByTabSwitch => "diagram_cache_skipped_by_tab_switch",
        }
    }
}

pub(crate) struct DiagramCacheMetrics;

impl DiagramCacheMetrics {
    pub(crate) fn emit(metric: DiagramCacheMetric, kind: &DiagramKind, content_checksum: &str) {
        tracing::debug!(
            metric = metric.as_str(),
            diagram_kind = kind.display_name(),
            content_checksum = content_checksum,
            "diagram svg cache"
        );
    }

    pub(crate) fn emit_pruned(kind_dir_name: &str, content_checksum: &str) {
        tracing::debug!(
            metric = DiagramCacheMetric::Pruned.as_str(),
            diagram_kind = kind_dir_name,
            content_checksum = content_checksum,
            "diagram svg cache"
        );
    }

    pub(crate) fn emit_tab_switch_skipped() {
        tracing::debug!(
            metric = DiagramCacheMetric::SkippedByTabSwitch.as_str(),
            "diagram svg cache"
        );
    }
}
