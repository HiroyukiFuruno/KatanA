use crate::shell::KatanaApp;
use egui_kittest::HarnessBuilder;
use std::path::Path;
use std::sync::Arc;

#[cfg(target_os = "macos")]
const NATIVE_FRAME_RATE: f32 = 60.0;

#[derive(Clone, Copy, Debug)]
pub(in crate::app::doc_close_latency_tests) enum ImageHarnessProfile {
    Memory,
    IsolatedDisk,
    #[cfg(target_os = "macos")]
    MetalDisk60Hz,
}

pub(super) fn builder(
    profile: ImageHarnessProfile,
    viewport: eframe::egui::Vec2,
) -> HarnessBuilder<KatanaApp> {
    let builder = egui_kittest::Harness::builder()
        .with_size(viewport)
        .with_pixels_per_point(1.0);
    if !matches!(profile, ImageHarnessProfile::Memory) {
        eprintln!("png_explorer_harness_profile profile={profile:?}");
    }
    match profile {
        ImageHarnessProfile::Memory | ImageHarnessProfile::IsolatedDisk => builder,
        #[cfg(target_os = "macos")]
        ImageHarnessProfile::MetalDisk60Hz => builder
            .with_step_dt(1.0 / NATIVE_FRAME_RATE)
            .with_os(eframe::egui::os::OperatingSystem::Mac)
            .with_pixels_per_point(2.0)
            .with_render_every_step(true)
            .renderer(metal_renderer()),
    }
}

pub(super) fn cache(
    profile: ImageHarnessProfile,
    settings_directory: &Path,
) -> Arc<dyn katana_platform::cache::CacheFacade> {
    if matches!(profile, ImageHarnessProfile::Memory) {
        return Arc::new(katana_platform::InMemoryCacheService::default());
    }
    let path = settings_directory.join("cache").join("cache.json");
    eprintln!("png_explorer_disk_cache isolated_path={}", path.display());
    Arc::new(katana_platform::DefaultCacheService::new(path))
}

#[cfg(target_os = "macos")]
fn metal_renderer() -> egui_kittest::wgpu::WgpuTestRenderer {
    use eframe::{egui_wgpu, wgpu};

    let mut setup = egui_kittest::wgpu::default_wgpu_setup();
    let egui_wgpu::WgpuSetup::CreateNew(configuration) = &mut setup else {
        panic!("Metal diagnostic requires an explicit new renderer");
    };
    /* WHY: CPU adapterへの暗黙代替を禁止し、実Metalでuploadと描画完了を検査する。 */
    configuration.instance_descriptor.backends = wgpu::Backends::METAL;
    configuration.native_adapter_selector = None;
    let state =
        egui_kittest::wgpu::create_render_state(setup, egui_wgpu::RendererOptions::default());
    let adapter = state.adapter.get_info();
    assert_eq!(
        adapter.backend,
        wgpu::Backend::Metal,
        "Metal backend is required"
    );
    assert_ne!(
        adapter.device_type,
        wgpu::DeviceType::Cpu,
        "CPU adapter is forbidden"
    );
    eprintln!(
        "png_explorer_gpu_profile layer=offscreen_upload_render_readback native_window=false backend={:?} device_type={:?} adapter={} predicted_hz=60 dpi=2",
        adapter.backend, adapter.device_type, adapter.name
    );
    egui_kittest::wgpu::WgpuTestRenderer::from_render_state(state)
}
