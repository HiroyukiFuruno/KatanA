#[cfg(any(windows, test))]
mod resize_events;
#[cfg(windows)]
mod windows_resize;

pub(super) struct NativeGraphicsOps;

impl NativeGraphicsOps {
    #[cfg(all(windows, not(test)))]
    pub(super) fn install_resize_filter(context: &eframe::CreationContext<'_>) {
        /* WHY: 最適化の登録失敗で起動まで失敗させず、既存経路を保って診断へ残す。 */
        if let Err(error) = windows_resize::WindowsResizeOps::install(context) {
            tracing::error!(event = "native_resize_filter_install_failed", %error);
        }
    }

    pub(super) fn configure_presentation(options: &mut eframe::NativeOptions) {
        /* WHY: Windowsの表示同期待ちを減らし、他OSの既定表示を保つ。 */
        #[cfg(not(target_os = "windows"))]
        let _ = options;

        #[cfg(target_os = "windows")]
        {
            options.wgpu_options.surface.present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn presentation_configuration_preserves_latency_and_other_defaults() {
        let expected = eframe::NativeOptions::default();
        let mut actual = expected.clone();
        #[cfg(target_os = "windows")]
        let expected_present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
        #[cfg(not(target_os = "windows"))]
        let expected_present_mode = expected.wgpu_options.surface.present_mode;
        super::NativeGraphicsOps::configure_presentation(&mut actual);

        assert_eq!(
            actual.wgpu_options.surface.present_mode,
            expected_present_mode
        );
        assert_eq!(
            actual.wgpu_options.surface.desired_maximum_frame_latency,
            Some(1)
        );
        assert_eq!(actual.centered, expected.centered);
        assert_eq!(actual.multisampling, expected.multisampling);
        assert_eq!(actual.depth_buffer, expected.depth_buffer);
        assert_eq!(actual.dithering, expected.dithering);
        assert_eq!(actual.run_and_return, expected.run_and_return);
    }
}
