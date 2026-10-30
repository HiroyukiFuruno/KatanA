use super::super::fixtures::{INITIAL_HARNESS_FRAME_STEPS, make_harness};
use super::LOAD_TIMEOUT;
use super::fixture::ExplorerImageOps;
use super::input::LoadingFixtureInputs;
use super::on_demand;
use std::path::Path;
use std::time::Duration;

#[test]
#[ignore = "requires explicit read-only workspace and PNG paths"]
fn supplied_png_reaches_mesh_in_original_and_single_image_directories() {
    compare_directories(measure);
}

#[test]
#[ignore = "requires explicit read-only workspace and PNG paths"]
fn supplied_png_reaches_mesh_with_on_demand_repaints_in_both_directories() {
    compare_directories(on_demand::measure);
}

fn compare_directories(measure: fn(&str, &Path, &Path) -> Duration) {
    let _render_lock = crate::test_render_env::RenderEnvLock::lock();
    let (root, image) = LoadingFixtureInputs::configured_workspace()
        .expect("supply KATANA_EXPLORER_WORKSPACE_ROOT and KATANA_EXPLORER_IMAGE_PATH");
    let single = tempfile::tempdir().expect("isolated single-image workspace");
    let single_image = single.path().join(image.file_name().expect("PNG filename"));
    std::fs::copy(&image, &single_image).expect("copy only selected PNG");
    assert!(
        std::fs::read(&image).expect("read original PNG")
            == std::fs::read(&single_image).expect("read PNG copy"),
        "comparison PNG bytes must match"
    );
    /* WHY: 元ディレクトリは読取専用で使い、比較側へ選択PNGだけコピーする。 */
    let single_elapsed = measure("single_image", single.path(), &single_image);
    let original_elapsed = measure("original", &root, &image);
    eprintln!(
        "png_explorer_directory_comparison original_ms={} single_image_ms={}",
        original_elapsed.as_secs_f64() * 1000.0,
        single_elapsed.as_secs_f64() * 1000.0
    );
}

fn measure(cohort: &str, root: &Path, image: &Path) -> Duration {
    let configuration = tempfile::tempdir().expect("isolated cohort settings");
    let mut harness = make_harness(&configuration);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        ExplorerImageOps::step(&mut harness);
    }
    eprintln!(
        "png_explorer_directory_cohort cohort={cohort} root={} image={}",
        root.display(),
        image.display()
    );
    ExplorerImageOps::open_workspace(&mut harness, root);
    let selected = ExplorerImageOps::click_png_row(&mut harness, image, None, 0);
    let elapsed = ExplorerImageOps::assert_texture_painted_with_budget(
        &mut harness,
        image,
        selected,
        LOAD_TIMEOUT,
    );
    eprintln!(
        "png_explorer_directory_result cohort={cohort} first_mesh_ms={}",
        elapsed.as_secs_f64() * 1000.0
    );
    ExplorerImageOps::close_all(&mut harness, 0, "PNG directory comparison");
    elapsed
}
