use super::defaults::SettingsDefaultOps;
use super::types::*;

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            last_workspace: None,
            open_tabs: vec![],
            active_tab_idx: None,
            ignored_directories: SettingsDefaultOps::default_ignored_directories(),
            max_depth: SettingsDefaultOps::default_max_depth(),
            visible_extensions: SettingsDefaultOps::default_visible_extensions(),
            extensionless_excludes: SettingsDefaultOps::default_extensionless_excludes(),
            restore_session: SettingsDefaultOps::default_restore_session(),
            enable_drawio_mount: SettingsDefaultOps::true_default(),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            cache_retention_days: SettingsDefaultOps::default_cache_retention(),
            optimize_for_speed: true,
            diagram_concurrency: SettingsDefaultOps::default_diagram_concurrency(),
        }
    }
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            pdf_engine: SettingsDefaultOps::default_pdf_engine(),
            html_template: SettingsDefaultOps::default_html_template(),
        }
    }
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_save_interval_secs: SettingsDefaultOps::default_auto_save_interval_secs(),
            auto_refresh: true,
            auto_refresh_interval_secs: SettingsDefaultOps::default_auto_refresh_interval_secs(),
            scroll_sync_enabled: true,
            confirm_close_dirty_tab: true,
            slideshow_hover_highlight: true,
            slideshow_show_diagram_controls: true,
        }
    }
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            toc_visible: true,
            toc_position: TocPosition::default(),
            split_direction: SplitDirection::default(),
            pane_order: PaneOrder::default(),
            sidebar_visible: true,
            toolbar_visible: true,
            status_bar_visible: true,
            active_pane_idx: 0,
            activity_rail_order: vec![
                ActivityRailItem::AddWorkspace,
                ActivityRailItem::WorkspaceToggle,
                ActivityRailItem::ExplorerToggle,
                ActivityRailItem::Search,
                ActivityRailItem::History,
            ],
            accordion_vertical_line: true,
            toc_default_visible: false,
            explorer_default_visible: true,
        }
    }
}

impl Default for IngestSettings {
    fn default() -> Self {
        Self {
            image_save_directory: SettingsDefaultOps::default_image_save_directory(),
            create_directory_if_not_exists: true,
            image_name_format: SettingsDefaultOps::default_image_name_format(),
            show_confirmation_dialog: true,
        }
    }
}
