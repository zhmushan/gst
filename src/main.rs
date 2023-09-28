use clap::Parser;
use error::AnyError;
use fetcher::{Fetcher, Site};

mod cache;
mod error;
mod fetcher;
mod git_utils;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    env_logger::init();

    let cli = Cli::parse();
    let fetcher = Fetcher::new(Site::GH, &cli.repo);
    fetcher.go(cli.project_name).await?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    repo: String,
    project_name: Option<String>,
}
