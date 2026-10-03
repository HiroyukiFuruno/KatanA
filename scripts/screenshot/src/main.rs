mod capture;
mod executor_harness;
mod fixture;
mod http_fixture;
mod request;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "katana-screenshot",
    about = "In-process KatanA UI screenshot runner (not packaged-app acceptance)"
)]
struct Cli {
    #[arg(long, value_name = "FILE", help = "Path to request JSON file")]
    request: PathBuf,
    #[arg(long, value_name = "DIR", help = "Output directory for PNG files")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let default_filter = tracing_subscriber::EnvFilter::new(
        "katana_ui=info,katana_core=info,katana_document_viewer=info",
    );
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(default_filter),
        )
        .try_init();
    let cli = Cli::parse();
    println!("[katana-screenshot] execution_mode=in_process; packaged_binary_tested=false");

    let request_path = cli
        .request
        .canonicalize()
        .with_context(|| format!("request file not found: {}", cli.request.display()))?;

    let req = request::load(&request_path)?;

    std::fs::create_dir_all(&cli.output)
        .with_context(|| format!("cannot create output dir: {}", cli.output.display()))?;
    let output_dir = cli.output.canonicalize()?;

    println!("[katana-screenshot] request: {}", req.name);
    println!("[katana-screenshot] output:  {}", output_dir.display());

    let tmp_dir = tempfile::Builder::new()
        .prefix("katana-screenshot-")
        // WHY: KatanA は workspace 復元時に system temp 配下を意図的に除外する。
        .tempdir_in(&output_dir)?;
    let fixture_env = fixture::setup(&req.fixture, tmp_dir.path())?;

    executor_harness::run(
        &req.steps,
        &req.fixture,
        &fixture_env.config_dir,
        fixture_env.workspace_dir.as_deref(),
        &output_dir,
    )?;

    println!("[katana-screenshot] done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::{Parser, error::ErrorKind};

    #[test]
    fn rejects_packaged_binary_instead_of_silently_testing_in_process() {
        for option in [
            vec!["--binary", "/tmp/KatanA"],
            vec!["--binary=/tmp/KatanA"],
        ] {
            let mut args = vec![
                "katana-screenshot",
                "--request",
                "input.json",
                "--output",
                "out",
            ];
            args.extend(option);
            let error = Cli::try_parse_from(args)
                .err()
                .expect("unsupported binary must fail");
            assert_eq!(error.kind(), ErrorKind::UnknownArgument);
        }
    }

    #[test]
    fn accepts_in_process_request_without_binary() {
        assert!(
            Cli::try_parse_from([
                "katana-screenshot",
                "--request",
                "input.json",
                "--output",
                "out"
            ])
            .is_ok()
        );
    }
}
