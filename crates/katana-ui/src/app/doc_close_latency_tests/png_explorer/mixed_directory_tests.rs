use super::super::fixtures::{
    INITIAL_HARNESS_FRAME_STEPS, ImageHarnessProfile, make_harness_with_profile,
};
use super::fixture::ExplorerImageOps;
use super::input::LoadingFixtureInputs;
use super::on_demand;
use crate::app::DocumentOps;
use egui_kittest::Harness;
use katana_core::workspace::TreeEntry;
use std::path::{Path, PathBuf};

#[test]
#[ignore = "requires explicit read-only workspace with six PNG and seven Office/HTML inputs"]
fn all_supplied_office_html_tabs_retain_six_png_on_demand_meshes() {
    run_profile(ImageHarnessProfile::Memory);
}

#[test]
#[ignore = "requires explicit read-only workspace with six PNG and seven Office/HTML inputs"]
fn all_supplied_tabs_retain_png_meshes_with_isolated_disk_cache() {
    run_profile(ImageHarnessProfile::IsolatedDisk);
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires real Metal GPU and explicit read-only workspace inputs"]
fn all_supplied_tabs_retain_png_meshes_with_metal_disk_cache_60hz() {
    run_profile(ImageHarnessProfile::MetalDisk60Hz);
}

fn run_profile(profile: ImageHarnessProfile) {
    let _render_lock = crate::test_render_env::RenderEnvLock::lock();
    let (root, selected_image) = LoadingFixtureInputs::configured_workspace()
        .expect("supply KATANA_EXPLORER_WORKSPACE_ROOT and KATANA_EXPLORER_IMAGE_PATH");
    let configuration = tempfile::tempdir().expect("isolated mixed workspace settings");
    let mut harness = make_harness_with_profile(&configuration, profile);
    for _ in 0..INITIAL_HARNESS_FRAME_STEPS {
        ExplorerImageOps::step(&mut harness);
    }
    ExplorerImageOps::open_workspace(&mut harness, &root);
    let (documents, images) = workspace_inputs(&harness, &selected_image);
    open_background_inputs(&mut harness, &documents);
    for (index, image) in images.iter().enumerate() {
        let elapsed = on_demand::click_and_await_mesh(&mut harness, image);
        assert_eq!(
            harness.state().state.document.open_documents.len(),
            documents.len() + index + 1
        );
        eprintln!(
            "png_explorer_mixed_png phase=initial_open path={} first_mesh_ms={} open_tabs={} lifecycle={:?}",
            image.display(),
            elapsed.as_secs_f64() * 1000.0,
            harness.state().state.document.open_documents.len(),
            harness.state().document_lifecycle_resources_for_test()
        );
    }
    revisit_with_all_tabs(&mut harness, &documents, &images);
    eprintln!(
        "png_explorer_mixed_stage stage=all_png_meshes_complete count={}",
        images.len()
    );
    eprintln!(
        "png_explorer_mixed_stage stage=close_start lifecycle={:?}",
        harness.state().document_lifecycle_resources_for_test()
    );
    ExplorerImageOps::close_all(&mut harness, 0, "mixed original workspace");
    eprintln!("png_explorer_mixed_stage stage=close_complete");
}

fn revisit_with_all_tabs(
    harness: &mut Harness<'_, crate::shell::KatanaApp>,
    documents: &[PathBuf],
    images: &[PathBuf],
) {
    let previous = documents.last().expect("Office tab before PNG selection");
    harness
        .state_mut()
        .handle_select_document(previous.clone(), true);
    on_demand::initialize_preview(harness, previous);
    for image in images {
        let elapsed = on_demand::click_and_await_mesh(harness, image);
        assert_eq!(
            harness.state().state.document.open_documents.len(),
            documents.len() + images.len()
        );
        eprintln!(
            "png_explorer_mixed_png phase=all_tabs_revisit path={} first_mesh_ms={} open_tabs={} lifecycle={:?}",
            image.display(),
            elapsed.as_secs_f64() * 1000.0,
            harness.state().state.document.open_documents.len(),
            harness.state().document_lifecycle_resources_for_test()
        );
    }
}

fn workspace_inputs(
    harness: &Harness<'_, crate::shell::KatanaApp>,
    selected_image: &Path,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let workspace = harness
        .state()
        .state
        .workspace
        .data
        .as_ref()
        .expect("loaded workspace");
    let mut documents = Vec::new();
    let mut images = Vec::new();
    for entry in &workspace.tree {
        let TreeEntry::File { path } = entry else {
            continue;
        };
        if entry.is_html() || office_order(path).is_some() {
            documents.push(path.clone());
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
        {
            images.push(path.clone());
        }
    }
    documents.sort_by_key(|path| {
        (
            office_order(path).unwrap_or(0),
            std::fs::metadata(path).expect("input metadata").len(),
            path.clone(),
        )
    });
    images.sort_by_key(|path| (path != selected_image, path.clone()));
    assert_eq!(
        documents.len(),
        7,
        "expected six Office inputs and one HTML"
    );
    assert_eq!(images.len(), 6, "expected all six supplied PNG inputs");
    assert!(images.iter().any(|path| path == selected_image));
    (documents, images)
}

fn office_order(path: &Path) -> Option<u8> {
    let extension = path.extension()?;
    if extension.eq_ignore_ascii_case("pptx") {
        Some(1)
    } else if extension.eq_ignore_ascii_case("xlsx") {
        Some(2)
    } else {
        None
    }
}

fn open_background_inputs(
    harness: &mut Harness<'_, crate::shell::KatanaApp>,
    documents: &[PathBuf],
) {
    for path in documents {
        eprintln!(
            "png_explorer_mixed_stage stage=open_input path={}",
            path.display()
        );
        harness
            .state_mut()
            .handle_select_document(path.clone(), true);
        /* WHY: 最初の描画完了は待たず、各形式の実surface開始まで進めて全タブを保持する。 */
        on_demand::initialize_preview(harness, path);
        let pane = &harness
            .state()
            .tab_previews
            .iter()
            .find(|preview| preview.path == *path)
            .expect("opened preview")
            .pane;
        eprintln!(
            "png_explorer_mixed_opened path={} intake={} surface={} document_frame={:?} html_frame={:?} failure={:?} lifecycle={:?}",
            path.display(),
            pane.document_intake.is_some(),
            pane.document_surface.is_some(),
            pane.document_frame_state_for_test(),
            pane.html_browser_frame_generation(),
            pane.document_failure_for_test(),
            harness.state().document_lifecycle_resources_for_test()
        );
    }
    assert_eq!(
        harness.state().state.document.open_documents.len(),
        documents.len()
    );
}
