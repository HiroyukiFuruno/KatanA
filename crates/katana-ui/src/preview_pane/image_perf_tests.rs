use super::ImagePerf;
use crate::preview_pane::local_image_loader::LocalImageLoader;
use crate::preview_pane::{ImageLogicOps, ViewerState};
use eframe::egui;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const CHILD: &str = "KATANA_IMAGE_PERF_CHILD";
const TEST: &str = "preview_pane::image_perf::tests::debug_controls_real_png_pipeline_stages";
const TRACE_IMAGE_WIDTH: u32 = 8;
const TRACE_IMAGE_HEIGHT: u32 = 8;
const TRACE_PIXEL: [u8; 4] = [19, 83, 167, u8::MAX];
const READY_TIMEOUT: Duration = Duration::from_secs(5);
const STABLE_FRAMES: u32 = 4;

#[test]
fn debug_controls_real_png_pipeline_stages() {
    if std::env::var_os(CHILD).is_some() {
        run_real_png();
        return;
    }
    for enabled in [false, true] {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, "1")
            .env("DEBUG", enabled.to_string())
            .output()
            .expect("isolated image trace child");
        assert!(
            output.status.success(),
            "child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let log = String::from_utf8(output.stderr).expect("UTF-8 trace");
        assert_eq!(log.contains("event=image_perf"), enabled, "{log}");
        if enabled {
            assert_stages(&log);
        }
    }
}

fn assert_stages(log: &str) {
    for stage in [
        "selection",
        "watch_queue_attempt",
        "watch_queued",
        "watch_start",
        "watch_end",
        "decode_queue_attempt",
        "decode_queued",
        "decode_start",
        "file_open_begin",
        "file_open_end",
        "read_begin",
        "read_end",
        "raster_decode_end",
        "decode_end",
        "result_collected",
        "texture_cpu_queued",
        "mesh_added",
    ] {
        assert!(
            log.contains(&format!("stage={stage} ")),
            "missing {stage}: {log}"
        );
    }
    assert!(log.contains("monotonic_us="));
    assert!(log.contains("owner=") && log.contains("generation=0 revision=0"));
    assert_eq!(
        transition_identities(log, "texture_cpu_queued"),
        transition_identities(log, "mesh_added")
    );
    assert!(log.contains("gpu_complete=false"));
}

fn transition_identities<'a>(log: &'a str, stage: &str) -> HashSet<&'a str> {
    let marker = format!("stage={stage} ");
    let mut identities = HashSet::new();
    for line in log.lines().filter(|line| line.contains(&marker)) {
        let (_, identity) = line
            .split_once(" path=")
            .expect("path and full request identity");
        /* WHY: revision更新による別textureは許容し、同一要求の重複通知を拒否する。 */
        assert!(identities.insert(identity), "duplicate {stage}: {line}");
    }
    assert!(!identities.is_empty(), "missing {stage}");
    identities
}

fn run_real_png() {
    let directory = tempfile::tempdir().expect("PNG fixture directory");
    let path = directory.path().join("trace-input.png");
    image::RgbaImage::from_pixel(
        TRACE_IMAGE_WIDTH,
        TRACE_IMAGE_HEIGHT,
        image::Rgba(TRACE_PIXEL),
    )
    .save(&path)
    .expect("real PNG");
    ImagePerf::selection(&path, true);
    let context = egui::Context::default();
    let loader = LocalImageLoader::default();
    let mut state = ViewerState::default();
    let deadline = Instant::now() + READY_TIMEOUT;
    let mut settled = 0;
    let mut previous_texture = None;
    while settled < STABLE_FRAMES {
        assert!(
            Instant::now() < deadline,
            "real PNG trace pipeline deadline"
        );
        loader.poll(context.cumulative_frame_nr());
        let mut output = context.run_ui(Default::default(), |ui| {
            let _ = ImageLogicOps::show_local_image(
                ui,
                &path,
                0,
                &loader,
                Some(&mut state),
                None,
                |_, _, _| {},
            );
        });
        if let Some(texture) = &state.texture {
            assert!(
                output
                    .shapes
                    .iter()
                    .any(|shape| has_texture(&shape.shape, texture.id()))
            );
            settled = if previous_texture == Some(texture.id()) {
                settled + 1
            } else {
                1
            };
            previous_texture = Some(texture.id());
        } else {
            settled = 0;
            previous_texture = None;
        }
        output.textures_delta.clear();
        std::thread::yield_now();
    }
}

fn has_texture(shape: &egui::epaint::Shape, texture: egui::TextureId) -> bool {
    match shape {
        egui::epaint::Shape::Mesh(mesh) => mesh.texture_id == texture,
        egui::epaint::Shape::Vec(shapes) => shapes.iter().any(|shape| has_texture(shape, texture)),
        _ => false,
    }
}
