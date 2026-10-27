pub(super) struct NativeGraphicsOps;

impl NativeGraphicsOps {
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
