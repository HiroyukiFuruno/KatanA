use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

const FIXTURE_SIDE: u32 = 180;
const FIXTURE_NOISE_SEED: u32 = 0x6d2b_79f5;
const FIXTURE_NOISE_SHIFTS: [u32; 3] = [13, 17, 5];

const REPAINT_TIMEOUT: Duration = Duration::from_secs(5);

fn large_png(path: &Path) -> u64 {
    let mut image = image::RgbImage::new(FIXTURE_SIDE, FIXTURE_SIDE);
    let mut state = FIXTURE_NOISE_SEED;
    for pixel in image.pixels_mut() {
        state ^= state << FIXTURE_NOISE_SHIFTS[0];
        state ^= state >> FIXTURE_NOISE_SHIFTS[1];
        state ^= state << FIXTURE_NOISE_SHIFTS[2];
        let bytes = state.to_le_bytes();
        *pixel = image::Rgb([bytes[0], bytes[1], bytes[2]]);
    }
    image.save(path).expect("large PNG fixture");
    std::fs::metadata(path).expect("PNG metadata").len()
}

#[test]
fn worker_completion_requests_repaint_and_reuses_unchanged_texture() {
    let _watch_guard = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().expect("fixture directory");
    let path = directory.path().join("about-100kb.png");
    let png_bytes = large_png(&path);
    assert!(
        (80_000..130_000).contains(&png_bytes),
        "fixture size: {png_bytes}"
    );

    let loader = LocalImageLoader::default();
    /* WHY: ファイル監視の通知を混ぜず、worker完了が再描画を要求することを確認する。 */
    loader
        .inner
        .watched_paths
        .lock()
        .expect("watched paths lock")
        .insert(path.clone());
    let context = egui::Context::default();
    let (repaint_tx, repaint_rx) = mpsc::channel();
    context.set_request_repaint_callback(move |info| {
        if info.delay.is_zero() {
            let _ = repaint_tx.send(Instant::now());
        }
    });

    let decode_requested_at = Instant::now();
    assert!(matches!(
        loader.texture(&context, &path, crate::theme_bridge::WHITE, 385),
        LocalTextureStatus::Pending
    ));

    /* WHY: 定期的な再描画で、完了通知の欠落を隠さない。 */
    let completion_wake_at = match repaint_rx.recv_timeout(REPAINT_TIMEOUT) {
        Ok(at) => at,
        Err(error) => {
            let result = loader
                .inner
                .result_rx
                .lock()
                .expect("result receiver lock")
                .try_recv();
            assert!(
                result.is_ok(),
                "worker did not publish a result before immediate repaint timeout ({error})"
            );
            panic!("worker published decode result but requested no immediate repaint");
        }
    };
    assert!(completion_wake_at.duration_since(decode_requested_at) < REPAINT_TIMEOUT);

    assert!(loader.poll(1));
    let LocalTextureStatus::Ready(texture) =
        loader.texture(&context, &path, crate::theme_bridge::WHITE, 385)
    else {
        panic!("decoded PNG should be ready after completion repaint");
    };
    let first_ready_at = Instant::now();
    eprintln!(
        "issue385 fixture={}B request_to_first_ready={:?} decode_request_to_wakeup={:?} wakeup_to_first_ready={:?}",
        png_bytes,
        first_ready_at.duration_since(decode_requested_at),
        completion_wake_at.duration_since(decode_requested_at),
        first_ready_at.duration_since(completion_wake_at),
    );
    assert_eq!(texture.size(), [180, 180]);

    for frame in 2..7 {
        assert!(!loader.poll(frame));
        let LocalTextureStatus::Ready(current) =
            loader.texture(&context, &path, crate::theme_bridge::WHITE, 385)
        else {
            panic!("unchanged image should remain ready");
        };
        assert_eq!(current.id(), texture.id());
    }
    assert!(!loader.is_pending(&RequestKey {
        path,
        background: crate::theme_bridge::WHITE,
    }));
}
