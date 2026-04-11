use crate::app_action::AppAction;
use crate::icon::{Icon, IconSize};
use crate::shell::KatanaApp;
use eframe::egui;

/* WHY: Constants for dashboard layout to avoid magic numbers and maintain consistency. */
const DASHBOARD_ITEM_SPACING: f32 = 4.0;
const DASHBOARD_PANEL_PADDING: f32 = 12.0;
const DASHBOARD_WELCOME_TITLE_SIZE: f32 = 32.0;
const DASHBOARD_SECTION_WIDTH: f32 = 320.0;
const DASHBOARD_COLUMNS_COUNT: usize = 2;
const DASHBOARD_SPACER_FACTOR_LARGE: f32 = 2.0;
const DASHBOARD_SPACER_FACTOR_HUGE: f32 = 3.0;
const RECENT_FILES_LIMIT: usize = 10;

/* WHY: Internal URIs for special documentation views. */
const URI_KATANA_GUIDE: &str = "Katana://Guide";
const URI_KATANA_CHANGELOG: &str = "Katana://ChangeLog";

/* WHY: Welcome dashboard view that provides quick access to recent files and essential guides. */
pub struct DashboardView;

impl DashboardView {
    /* WHY: Factory method to create a new instance of DashboardView. */
    pub(crate) fn new(_app: &mut KatanaApp) -> Self {
        Self
    }

    /* WHY: Main rendering entry point for the dashboard panel. */
    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut KatanaApp) {
        let i18n = crate::i18n::I18nOps::get();
        let margin = DASHBOARD_PANEL_PADDING;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ui.visuals().panel_fill))
            .show_inside(ui, |ui| {
                ui.add_space(margin * DASHBOARD_SPACER_FACTOR_LARGE);

                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(&i18n.dashboard.welcome_title)
                            .heading()
                            .strong()
                            .size(DASHBOARD_WELCOME_TITLE_SIZE),
                    );
                    ui.add_space(DASHBOARD_ITEM_SPACING * DASHBOARD_SPACER_FACTOR_LARGE);
                    ui.label(
                        egui::RichText::new(&i18n.dashboard.welcome_subtitle)
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(margin * DASHBOARD_SPACER_FACTOR_HUGE);

                    Self::render_sections(ui, app);
                });
            });
    }

    /* WHY: Organizes the dashboard layout into columns for actions and recent files. */
    fn render_sections(ui: &mut egui::Ui, app: &mut KatanaApp) {
        ui.columns(DASHBOARD_COLUMNS_COUNT, |columns| {
            /* WHY: Left Column displays action buttons like New File and Guides. */
            columns[0].vertical(|ui| {
                ui.set_max_width(DASHBOARD_SECTION_WIDTH);
                Self::render_start_section(ui, app);
            });

            /* WHY: Right Column displays a list of recently used documents. */
            columns[1].vertical(|ui| {
                ui.set_max_width(DASHBOARD_SECTION_WIDTH);
                Self::render_recent_section(ui, app);
            });
        });
    }

    /* WHY: Renders a dashboard item with an icon and label using AlignCenter for linter-compliant horizontal layout. */
    fn render_dashboard_item(ui: &mut egui::Ui, icon: Icon, label: &str) -> egui::Response {
        /* WHY: AlignCenter itself returns a response that represents the union of its parts. */
        crate::widgets::AlignCenter::new()
            .left(|ui| ui.add(icon.button(ui, IconSize::Medium)))
            .left(|ui| ui.button(label))
            .show(ui)
    }

    /* WHY: Renders the primary action buttons (New File, Guide, ChangeLog) in the "Start" section. */
    fn render_start_section(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let i18n = crate::i18n::I18nOps::get();

        ui.label(egui::RichText::new(&i18n.dashboard.start_section).strong());
        ui.add_space(DASHBOARD_ITEM_SPACING);

        if Self::render_dashboard_item(ui, Icon::Plus, &i18n.action.new_file).clicked() {
            app.pending_action = AppAction::PickOpenWorkspace;
        }

        if Self::render_dashboard_item(ui, Icon::Help, &i18n.menu.help).clicked() {
            app.pending_action =
                AppAction::SelectDocument(std::path::PathBuf::from(URI_KATANA_GUIDE));
        }

        if Self::render_dashboard_item(ui, Icon::Rocket, &i18n.menu.release_notes).clicked() {
            app.pending_action =
                AppAction::SelectDocument(std::path::PathBuf::from(URI_KATANA_CHANGELOG));
        }
    }

    /* WHY: Displays list of recently accessed markdown files with quick-open navigation. */
    fn render_recent_section(ui: &mut egui::Ui, app: &mut KatanaApp) {
        let i18n = crate::i18n::I18nOps::get();

        ui.label(egui::RichText::new(&i18n.dashboard.recent_section).strong());
        ui.add_space(DASHBOARD_ITEM_SPACING);

        let recent_files = app
            .state
            .config
            .settings
            .settings()
            .search
            .recent_md_queries
            .clone();
        if recent_files.is_empty() {
            ui.label(
                egui::RichText::new(&i18n.dashboard.no_recent_files)
                    .color(ui.visuals().weak_text_color())
                    .italics(),
            );
        } else {
            for term in recent_files.iter().take(RECENT_FILES_LIMIT) {
                if Self::render_dashboard_item(ui, Icon::Document, term).clicked() {
                    app.pending_action = AppAction::SelectDocument(std::path::PathBuf::from(term));
                }
            }
        }
    }
}
