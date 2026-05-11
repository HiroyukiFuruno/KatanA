use super::types::*;
use crate::markdown::color_preset::DiagramColorPreset;

impl DiagramBackendId {
    pub fn new(language: DiagramBackendLanguage, implementation: impl Into<String>) -> Self {
        Self {
            language,
            implementation: implementation.into(),
        }
    }
}

impl DiagramBackendVersion {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            runtime_version: value.clone(),
            renderer_profile: value.clone(),
            value,
        }
    }

    pub fn from_kcf(
        crate_version: &str,
        runtime_name: &str,
        runtime_version: &str,
        runtime_checksum: &str,
        renderer_profile: &str,
    ) -> Self {
        let runtime = format!("{runtime_name}:{runtime_version};checksum={runtime_checksum}");
        Self {
            value: format!(
                "crate=katana-canvas-forge:{crate_version};runtime={runtime};profile={renderer_profile}"
            ),
            runtime_version: runtime,
            renderer_profile: renderer_profile.to_string(),
        }
    }
}

impl DiagramThemeSnapshot {
    pub fn current() -> Self {
        let is_dark = DiagramColorPreset::is_dark_mode();
        Self::from_preset(
            if is_dark { "dark" } else { "light" },
            is_dark,
            DiagramColorPreset::current(),
        )
    }

    pub fn from_preset(
        name: impl Into<String>,
        is_dark: bool,
        preset: &DiagramColorPreset,
    ) -> Self {
        Self {
            name: name.into(),
            is_dark,
            background: preset.background.to_string(),
            text: preset.text.to_string(),
            fill: preset.fill.to_string(),
            stroke: preset.stroke.to_string(),
            arrow: preset.arrow.to_string(),
            drawio_label_color: preset.drawio_label_color.to_string(),
            mermaid_theme: preset.mermaid_theme.to_string(),
            plantuml_class_background: preset.plantuml_class_bg.to_string(),
            plantuml_note_background: preset.plantuml_note_bg.to_string(),
            plantuml_note_text: preset.plantuml_note_text.to_string(),
            syntax_theme_dark: preset.syntax_theme_dark.to_string(),
            syntax_theme_light: preset.syntax_theme_light.to_string(),
            preview_text: preset.preview_text.to_string(),
        }
    }

    pub fn fingerprint(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| self.name.clone())
    }

    pub fn render_policy_fingerprint(&self) -> String {
        format!(
            "background={};cacheProfile={};dark={}",
            self.background, self.name, self.is_dark
        )
    }
}

impl DiagramDocumentContext {
    pub fn cache_id(&self) -> String {
        match self {
            Self::WorkspaceFile {
                workspace_root,
                document_path,
            } => format!(
                "{}:{}",
                workspace_root.to_string_lossy(),
                document_path.to_string_lossy()
            ),
            Self::Detached { display_name } => display_name.clone(),
        }
    }
}

impl DiagramBackendCacheKey {
    pub fn new(
        backend_id: DiagramBackendId,
        backend_version: DiagramBackendVersion,
        input: &DiagramBackendInput,
    ) -> Self {
        Self {
            runtime_version: backend_version.runtime_version.clone(),
            renderer_profile: backend_version.renderer_profile.clone(),
            backend_id,
            backend_version,
            language: input.language.clone(),
            source: input.source.clone(),
            options: input.options.clone(),
            render_config: input.options.fingerprint(),
            render_policy: input.theme.render_policy_fingerprint(),
            theme_fingerprint: input.theme.fingerprint(),
            theme: input.theme.clone(),
        }
    }
}

impl DiagramRenderOptions {
    fn fingerprint(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
