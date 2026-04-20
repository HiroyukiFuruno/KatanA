mod capture;
mod executor;
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
    #[arg(long, value_name = "PATH", help = "Path to the KatanA binary")]
    binary: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let request_path = cli.request.canonicalize()
        .with_context(|| format!("request file not found: {}", cli.request.display()))?;
    let binary_path = cli.binary.canonicalize()
        .with_context(|| format!("KatanA binary not found: {}", cli.binary.display()))?;

    let req = request::load(&request_path)?;

    std::fs::create_dir_all(&cli.output)
        .with_context(|| format!("cannot create output dir: {}", cli.output.display()))?;
    let output_dir = cli.output.canonicalize()?;

    println!("[katana-screenshot] request: {}", req.name);
    println!("[katana-screenshot] output:  {}", output_dir.display());

    let tmp_dir = tempfile::Builder::new().prefix("katana-screenshot-").tempdir()?;
    let env = fixture::setup(&req.fixture, tmp_dir.path())?;

    executor::run(&req.steps, &output_dir, &binary_path, &env.home_dir)?;

    println!("[katana-screenshot] done");
    Ok(())
}
