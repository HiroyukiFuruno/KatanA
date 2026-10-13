#![cfg(unix)]

use super::{LocalImageLoader, LocalImageStatus};
use eframe::egui;
use katana_core::system::ProcessService;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

const CHILD_ENV: &str = "KATANA_LOCAL_IMAGE_FIFO_CHILD";
const TEST_BACKGROUND: egui::Color32 = crate::theme_bridge::WHITE;
const WAIT_SECONDS: u64 = 5;
const FOLLOWUP_PIXEL: [u8; 4] = [0, 255, 0, 255];
const SOURCE_PIXEL: [u8; 4] = [255, 0, 0, 255];

#[test]
fn reset_does_not_leave_fifo_decode_blocking_followup() {
    let child_test = format!(
        "{}::fifo_decode_child",
        module_path!()
            .strip_prefix("katana_ui::")
            .expect("module path")
    );
    let executable = std::env::current_exe().expect("test executable");
    let output = ProcessService::create_command(executable.to_str().expect("executable path"))
        .args(["--exact", &child_test, "--nocapture"])
        .env(CHILD_ENV, "1")
        .output()
        .expect("spawn FIFO child test");
    assert!(String::from_utf8_lossy(&output.stdout).contains("running 1 test"));
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn fifo_decode_child() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let directory = tempfile::tempdir().expect("fixture directory");
    let source = directory.path().join("blocked.png");
    let payload = directory.path().join("payload.png");
    let next = directory.path().join("next.png");
    make_fifo(&source);
    let loader = LocalImageLoader::default();
    request_until_pending(&loader, &source);
    let mut writer = open_fifo_writer(&loader, &source);
    loader.reset();
    write_png_payload(&mut writer, &payload);

    image::RgbaImage::from_pixel(1, 1, image::Rgba(FOLLOWUP_PIXEL))
        .save(&next)
        .expect("follow-up PNG");
    assert_followup_ready(&loader, &next);
}

fn make_fifo(path: &Path) {
    assert!(
        ProcessService::create_command("mkfifo")
            .arg(path)
            .status()
            .expect("mkfifo")
            .success()
    );
}

fn request_until_pending(loader: &LocalImageLoader, path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    while Instant::now() < deadline {
        loader.poll(0);
        if matches!(
            loader.request(path, TEST_BACKGROUND),
            LocalImageStatus::Pending
        ) {
            return;
        }
        std::thread::yield_now();
    }
    panic!("FIFO decode did not enter Pending");
}

fn open_fifo_writer(loader: &LocalImageLoader, path: &Path) -> std::fs::File {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    while Instant::now() < deadline {
        loader.poll(0);
        let _ = loader.request(path, TEST_BACKGROUND);
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(writer) => return writer,
            Err(error) if error.raw_os_error() == Some(libc::ENXIO) => {
                std::thread::yield_now();
            }
            Err(error) => panic!("open FIFO writer: {error}"),
        }
    }
    panic!("FIFO reader did not reach the real decoder");
}

fn write_png_payload(writer: &mut std::fs::File, fixture: &Path) {
    image::RgbaImage::from_pixel(1, 1, image::Rgba(SOURCE_PIXEL))
        .save(fixture)
        .expect("source PNG");
    let bytes = std::fs::read(fixture).expect("source bytes");
    if let Err(error) = writer.write_all(&bytes) {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }
}

fn assert_followup_ready(loader: &LocalImageLoader, path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(WAIT_SECONDS);
    while Instant::now() < deadline {
        loader.poll(0);
        if let LocalImageStatus::Ready(image) = loader.request(path, TEST_BACKGROUND) {
            assert_eq!(image.size, [1, 1]);
            assert_eq!(image.pixels.len(), 1);
            assert_eq!(image.pixels[0].to_array(), FOLLOWUP_PIXEL);
            return;
        }
        std::thread::yield_now();
    }
    panic!("follow-up PNG remained blocked behind stale FIFO decode");
}
