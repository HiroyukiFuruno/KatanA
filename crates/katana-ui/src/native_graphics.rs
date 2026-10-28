#[cfg(any(windows, test))]
mod resize_events;
#[cfg(windows)]
mod windows_resize;

pub(super) struct NativeGraphicsOps;

impl NativeGraphicsOps {
    #[cfg(all(windows, not(test)))]
    pub(super) fn install_resize_creation_hook() -> Option<windows_resize::creation::CreationHook> {
        match windows_resize::creation::CreationHook::install() {
            Ok(hook) => Some(hook),
            Err(error) => {
                tracing::error!(event = "native_resize_creation_hook_install_failed", %error);
                None
            }
        }
    }

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
            Self::configure_dx12_wait(
                options,
                eframe::wgpu::Dx12UseFrameLatencyWaitableObject::from_env(),
            );
        }
    }

    #[cfg(any(windows, test))]
    fn configure_dx12_wait(
        options: &mut eframe::NativeOptions,
        requested: Option<eframe::wgpu::Dx12UseFrameLatencyWaitableObject>,
    ) {
        let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup
        else {
            return;
        };
        /* WHY: GPUのキュー上限を保ったままUI threadでの追加待機を避け、明示設定は尊重する。 */
        setup
            .instance_descriptor
            .backend_options
            .dx12
            .latency_waitable_object =
            requested.unwrap_or(eframe::wgpu::Dx12UseFrameLatencyWaitableObject::DontWait);
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

    #[test]
    fn dx12_default_avoids_extra_wait_and_preserves_surface_and_adapter_configuration() {
        let expected = eframe::NativeOptions::default();
        let mut actual = expected.clone();
        super::NativeGraphicsOps::configure_dx12_wait(&mut actual, None);
        let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &actual.wgpu_options.wgpu_setup else {
            panic!("the default setup creates the instance");
        };
        let eframe::egui_wgpu::WgpuSetup::CreateNew(original) = &expected.wgpu_options.wgpu_setup
        else {
            panic!("the default setup creates the instance");
        };
        assert!(matches!(
            setup
                .instance_descriptor
                .backend_options
                .dx12
                .latency_waitable_object,
            eframe::wgpu::Dx12UseFrameLatencyWaitableObject::DontWait
        ));
        assert_eq!(
            setup.instance_descriptor.backends,
            original.instance_descriptor.backends
        );
        assert_eq!(
            setup.instance_descriptor.flags,
            original.instance_descriptor.flags
        );
        assert_eq!(
            format!(
                "{:?}",
                setup
                    .instance_descriptor
                    .backend_options
                    .dx12
                    .shader_compiler
            ),
            format!(
                "{:?}",
                original
                    .instance_descriptor
                    .backend_options
                    .dx12
                    .shader_compiler
            )
        );
        assert_eq!(
            actual.wgpu_options.surface.present_mode,
            expected.wgpu_options.surface.present_mode
        );
        assert_eq!(
            actual.wgpu_options.surface.desired_maximum_frame_latency,
            Some(1)
        );
    }

    #[test]
    fn dx12_explicit_wait_options_remain_authoritative() {
        use eframe::wgpu::Dx12UseFrameLatencyWaitableObject;
        for requested in [
            Dx12UseFrameLatencyWaitableObject::Wait,
            Dx12UseFrameLatencyWaitableObject::DontWait,
            Dx12UseFrameLatencyWaitableObject::None,
        ] {
            let mut options = eframe::NativeOptions::default();
            let expected = std::mem::discriminant(&requested);
            super::NativeGraphicsOps::configure_dx12_wait(&mut options, Some(requested));
            let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &options.wgpu_options.wgpu_setup
            else {
                panic!("the default setup creates the instance");
            };
            assert_eq!(
                std::mem::discriminant(
                    &setup
                        .instance_descriptor
                        .backend_options
                        .dx12
                        .latency_waitable_object
                ),
                expected
            );
            assert_eq!(
                options.wgpu_options.surface.desired_maximum_frame_latency,
                Some(1)
            );
        }
    }
}
