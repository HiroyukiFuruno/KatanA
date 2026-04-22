mod executor_harness;
mod fixture;
mod request;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "katana-screenshot", about = "Generic screenshot runner for KatanA")]
struct Cli {
    #[arg(long, value_name = "FILE", help = "Path to request JSON file")]
    request: PathBuf,
    #[arg(long, value_name = "DIR", help = "Output directory for PNG files")]
    output: PathBuf,
    #[arg(
        long,
        value_name = "PATH",
        help = "Ignored (kept for backward compatibility)"
    )]
    binary: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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
        .tempdir()?;
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
