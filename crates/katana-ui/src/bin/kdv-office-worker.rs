#![deny(warnings, clippy::all)]

fn main() {
    std::process::exit(katana_document_viewer::OfficeWorkerEntrypoint::run_from_env());
}
