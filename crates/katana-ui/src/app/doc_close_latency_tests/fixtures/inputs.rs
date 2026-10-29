use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(in crate::app::doc_close_latency_tests) struct Fixture {
    pub(in crate::app::doc_close_latency_tests) kind: &'static str,
    pub(in crate::app::doc_close_latency_tests) path: PathBuf,
}

fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::copy(source, destination).unwrap_or_else(|error| {
        panic!(
            "failed to copy fixture {} to {}: {error}",
            source.display(),
            destination.display()
        )
    });
}

pub(in crate::app::doc_close_latency_tests) fn fixtures(directory: &Path) -> Vec<Fixture> {
    let html_source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/screenshot/fixtures/v0-22-36-html-browser");
    let office_source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/screenshot/fixtures/v0-22-38-multi-format");
    let html_directory = directory.join("html");
    let office_directory = directory.join("office");
    std::fs::create_dir_all(&html_directory).expect("create isolated HTML fixture directory");
    std::fs::create_dir_all(&office_directory).expect("create isolated Office fixture directory");

    let html = html_directory.join("index.html");
    copy_fixture(&html_source.join("index.html"), &html);
    copy_fixture(
        &html_source.join("linked-panel.html"),
        &html_directory.join("linked-panel.html"),
    );
    let docx = office_directory.join("representative.docx");
    let xlsx = office_directory.join("representative.xlsx");
    let pptx = office_directory.join("representative.pptx");
    for (source, destination) in [
        ("representative.docx", &docx),
        ("representative.xlsx", &xlsx),
        ("representative.pptx", &pptx),
    ] {
        copy_fixture(&office_source.join(source), destination);
    }
    vec![
        Fixture {
            kind: "html",
            path: html,
        },
        Fixture {
            kind: "docx",
            path: docx,
        },
        Fixture {
            kind: "xlsx",
            path: xlsx,
        },
        Fixture {
            kind: "pptx",
            path: pptx,
        },
    ]
}
