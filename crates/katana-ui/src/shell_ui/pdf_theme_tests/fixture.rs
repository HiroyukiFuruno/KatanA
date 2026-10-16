use crate::app::export_poll::ExportPoll;
use crate::app_state::AppState;
use crate::shell::KatanaApp;
use katana_core::markdown::DiagramThemeSnapshot;
use katana_platform::theme::{Rgb, ThemeColors, ThemePreset};
use katana_platform::{InMemoryCacheService, SettingsService};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const PDF_MARKDOWN: &str = "# PDF Theme Check\n\nBODY TEXT CONTRAST SAMPLE 0123456789\n\n| THEME TOKEN | VALUE |\n| --- | --- |\n| HEADER ROW | PRESENT |\n| STRIPE ROW | PRESENT |\n\n```mermaid\nflowchart LR\n  A[Diagram source] --> B[Diagram result]\n  classDef acceptance fill:#D22B5F,stroke:#271D2A,color:#FFFFFF\n  class A,B acceptance\n```\n";
const PDF_EXPORT_DEADLINE_SECONDS: u64 = 90;
const THEME_CASE_COUNT: usize = 4;
const DARK_BACKGROUND: Rgb = Rgb {
    r: 16,
    g: 24,
    b: 32,
};
const DARK_TEXT: Rgb = Rgb {
    r: 246,
    g: 242,
    b: 232,
};
const DARK_BORDER: Rgb = Rgb {
    r: 78,
    g: 99,
    b: 118,
};
const DARK_SELECTION: Rgb = Rgb {
    r: 20,
    g: 120,
    b: 150,
};
const LIGHT_BACKGROUND: Rgb = Rgb {
    r: 250,
    g: 246,
    b: 236,
};
const LIGHT_TEXT: Rgb = Rgb {
    r: 29,
    g: 43,
    b: 54,
};
const LIGHT_BORDER: Rgb = Rgb {
    r: 92,
    g: 122,
    b: 110,
};
const LIGHT_SELECTION: Rgb = Rgb {
    r: 112,
    g: 166,
    b: 194,
};

pub(super) struct ThemeCase {
    pub name: &'static str,
    preset: ThemePreset,
    custom_colors: Option<ThemeColors>,
}

pub(super) struct PdfThemeFixture;

impl PdfThemeFixture {
    pub(super) fn theme_cases() -> [ThemeCase; THEME_CASE_COUNT] {
        [
            custom_case("custom-dark", ThemePreset::KatanaDark, dark_custom_colors()),
            custom_case(
                "custom-light",
                ThemePreset::KatanaLight,
                light_custom_colors(),
            ),
            preset_case("preset-dark", ThemePreset::KatanaDark),
            preset_case("preset-light", ThemePreset::KatanaLight),
        ]
    }

    pub(super) fn create_artifact_directory() -> PathBuf {
        let output_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp/issue312-pdf-theme-2026-10-13");
        std::fs::create_dir_all(&output_root).expect("create retained PDF evidence directory");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let run_directory = output_root.join(format!("{}-{nonce}", std::process::id()));
        std::fs::create_dir(&run_directory).expect("create unique PDF evidence run directory");
        run_directory
    }

    pub(super) fn export_pdf(
        case: &ThemeCase,
        output_directory: &Path,
    ) -> (PathBuf, DiagramThemeSnapshot) {
        let mut app = make_app();
        let configured = set_app_theme(&mut app, case);
        let theme = DiagramThemeSnapshot::current();
        super::expected::ExpectedTheme::assert_snapshot(&configured, &theme);
        let source_path = output_directory.join(format!("{}.md", case.name));
        let pdf_path = output_directory.join(format!("{}.pdf", case.name));
        write_fixture(&source_path);
        start_export(&mut app, &source_path, &pdf_path);
        let result = receive_export(&app);
        assert_exported_pdf(&result, &pdf_path);

        (result, theme)
    }
}

fn write_fixture(source_path: &Path) {
    std::fs::write(source_path, PDF_MARKDOWN).expect("write ASCII Markdown fixture");
}

fn start_export(app: &mut KatanaApp, source_path: &Path, pdf_path: &Path) {
    ExportPoll::perform_tool_export(
        app,
        PDF_MARKDOWN,
        "pdf",
        pdf_path.to_path_buf(),
        source_path,
    );
}

fn receive_export(app: &KatanaApp) -> PathBuf {
    app.export_tasks[0]
        .rx
        .recv_timeout(std::time::Duration::from_secs(PDF_EXPORT_DEADLINE_SECONDS))
        .expect("PDF export must finish within the bounded deadline")
        .expect("PDF export pipeline must succeed")
}

fn assert_exported_pdf(result: &Path, requested_path: &Path) {
    assert_eq!(
        result, requested_path,
        "export must write the requested evidence PDF"
    );
    assert!(
        result.is_file(),
        "PDF export must create {}",
        result.display()
    );
}

fn make_app() -> KatanaApp {
    let state = AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        SettingsService::default(),
        Arc::new(InMemoryCacheService::default()),
    );
    KatanaApp::new(state)
}

fn set_app_theme(app: &mut KatanaApp, case: &ThemeCase) -> ThemeColors {
    let settings = app.state.config.settings.settings_mut();
    settings.theme.preset = case.preset;
    settings.theme.custom_color_overrides = case.custom_colors.clone();
    let colors = settings.effective_theme_colors();
    app.apply_theme(&egui::Context::default(), &colors);
    colors
}

fn preset_case(name: &'static str, preset: ThemePreset) -> ThemeCase {
    ThemeCase {
        name,
        preset,
        custom_colors: None,
    }
}

fn custom_case(name: &'static str, preset: ThemePreset, colors: ThemeColors) -> ThemeCase {
    ThemeCase {
        name,
        preset,
        custom_colors: Some(colors),
    }
}

fn dark_custom_colors() -> ThemeColors {
    let mut colors = ThemePreset::KatanaDark.colors();
    colors.name = "Acceptance Custom Dark".to_string();
    colors.preview.background = DARK_BACKGROUND;
    colors.preview.text = DARK_TEXT;
    colors.preview.border = DARK_BORDER;
    colors.preview.selection = DARK_SELECTION;
    colors
}

fn light_custom_colors() -> ThemeColors {
    let mut colors = ThemePreset::KatanaLight.colors();
    colors.name = "Acceptance Custom Light".to_string();
    colors.preview.background = LIGHT_BACKGROUND;
    colors.preview.text = LIGHT_TEXT;
    colors.preview.border = LIGHT_BORDER;
    colors.preview.selection = LIGHT_SELECTION;
    colors
}
