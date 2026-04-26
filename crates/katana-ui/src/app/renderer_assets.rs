use crate::app_state::AppAction;
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
    fn poll_renderer_asset_bootstrap(&mut self, ctx: &egui::Context);
}

impl RendererAssetOps for KatanaApp {
    fn start_renderer_asset_bootstrap(&mut self) {
        let assets = missing_renderer_assets();
        if assets.is_empty() {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.renderer_asset_rx = Some(rx);
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

    fn poll_renderer_asset_bootstrap(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.renderer_asset_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(downloaded)) => {
                if !downloaded.is_empty() {
                    self.pending_action = AppAction::RefreshDiagrams;
                }
                self.renderer_asset_rx = None;
            }
            Ok(Err(error)) => {
                tracing::warn!("Renderer asset update failed: {}", error);
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

fn missing_renderer_assets() -> Vec<RendererAsset> {
    let mut assets = Vec::new();

    if katana_core::markdown::drawio_renderer::DrawioRendererOps::find_drawio_js().is_none()
        && let Some(dest) =
            katana_core::markdown::drawio_renderer::DrawioRendererOps::default_install_path()
    {
        assets.push(RendererAsset {
            tool_name: "Draw.io",
            url: DRAWIO_DOWNLOAD_URL,
            dest,
        });
    }

    if katana_core::markdown::mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none()
        && let Some(dest) =
            katana_core::markdown::mermaid_renderer::MermaidBinaryOps::default_install_path()
    {
        assets.push(RendererAsset {
            tool_name: "Mermaid",
            url: MERMAID_DOWNLOAD_URL,
            dest,
        });
    }

    assets
}
