use crate::app_state::{AppAction, StatusType};
use crate::shell::KatanaApp;

pub(crate) const DRAWIO_DOWNLOAD_URL: &str = "https://viewer.diagrams.net/js/viewer-static.min.js";
pub(crate) const MERMAID_DOWNLOAD_URL: &str =
    "https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js";

struct RendererAsset {
    tool_name: &'static str,
    url: &'static str,
    dest: std::path::PathBuf,
}

pub(crate) trait RendererAssetOps {
    fn start_renderer_asset_bootstrap(&mut self);
    fn start_renderer_asset_repair(&mut self);
    fn poll_renderer_asset_bootstrap(&mut self, ctx: &egui::Context);
}

impl RendererAssetOps for KatanaApp {
    fn start_renderer_asset_bootstrap(&mut self) {
        self.start_renderer_asset_download(missing_renderer_assets());
    }

    fn start_renderer_asset_repair(&mut self) {
        self.start_renderer_asset_download(all_renderer_assets());
    }

    fn poll_renderer_asset_bootstrap(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.renderer_asset_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(downloaded)) => {
                if !downloaded.is_empty() {
                    self.state.layout.status_message = Some((
                        crate::i18n::I18nOps::tf(
                            &crate::i18n::I18nOps::get().plantuml.tool_installed,
                            &[("tool", &downloaded.join(" / "))],
                        ),
                        StatusType::Success,
                    ));
                    self.pending_action = AppAction::RefreshDiagrams;
                }
                self.renderer_asset_rx = None;
            }
            Ok(Err(error)) => {
                tracing::warn!("Renderer asset update failed: {}", error);
                self.state.layout.status_message = Some((
                    format!(
                        "{}{}",
                        crate::i18n::I18nOps::get().plantuml.download_error,
                        error
                    ),
                    StatusType::Error,
                ));
                self.renderer_asset_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                ctx.request_repaint_after(std::time::Duration::from_millis(
                    crate::shell::DOWNLOAD_STATUS_CHECK_INTERVAL_MS,
                ));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.renderer_asset_rx = None;
            }
        }
    }
}

trait RendererAssetDownloadOps {
    fn start_renderer_asset_download(&mut self, assets: Vec<RendererAsset>);
}

impl RendererAssetDownloadOps for KatanaApp {
    fn start_renderer_asset_download(&mut self, assets: Vec<RendererAsset>) {
        if assets.is_empty() {
            return;
        }
        if self.renderer_asset_rx.is_some() {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.renderer_asset_rx = Some(rx);
        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().plantuml.downloading_tool,
                &[("tool", "Draw.io / Mermaid")],
            ),
            StatusType::Info,
        ));
        std::thread::spawn(move || {
            let mut downloaded = Vec::new();
            for asset in assets {
                if let Err(error) =
                    katana_core::system::ProcessService::download_file(asset.url, &asset.dest)
                {
                    let _ = tx.send(Err(error));
                    return;
                }
                downloaded.push(asset.tool_name.to_string());
            }
            let _ = tx.send(Ok(downloaded));
        });
    }
}

fn missing_renderer_assets() -> Vec<RendererAsset> {
    let mut assets = Vec::new();

    if katana_core::markdown::DiagramRuntimeAssetOps::find_path(
        katana_core::markdown::DiagramRuntimeAssetKind::DrawIo,
    )
    .is_none()
        && let Some(dest) = katana_core::markdown::DiagramRuntimeAssetOps::resolve_path(
            katana_core::markdown::DiagramRuntimeAssetKind::DrawIo,
        )
    {
        assets.push(RendererAsset {
            tool_name: "Draw.io",
            url: DRAWIO_DOWNLOAD_URL,
            dest,
        });
    }

    if katana_core::markdown::DiagramRuntimeAssetOps::find_path(
        katana_core::markdown::DiagramRuntimeAssetKind::Mermaid,
    )
    .is_none()
        && let Some(dest) = katana_core::markdown::DiagramRuntimeAssetOps::resolve_path(
            katana_core::markdown::DiagramRuntimeAssetKind::Mermaid,
        )
    {
        assets.push(RendererAsset {
            tool_name: "Mermaid",
            url: MERMAID_DOWNLOAD_URL,
            dest,
        });
    }

    assets
}

fn all_renderer_assets() -> Vec<RendererAsset> {
    [
        renderer_asset(
            katana_core::markdown::DiagramRuntimeAssetKind::DrawIo,
            "Draw.io",
            DRAWIO_DOWNLOAD_URL,
        ),
        renderer_asset(
            katana_core::markdown::DiagramRuntimeAssetKind::Mermaid,
            "Mermaid",
            MERMAID_DOWNLOAD_URL,
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn renderer_asset(
    kind: katana_core::markdown::DiagramRuntimeAssetKind,
    tool_name: &'static str,
    url: &'static str,
) -> Option<RendererAsset> {
    katana_core::markdown::DiagramRuntimeAssetOps::resolve_path(kind).map(|dest| RendererAsset {
        tool_name,
        url,
        dest,
    })
}
