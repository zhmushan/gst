use clap::Parser;
use config::Config;
use error::AnyError;
use fetcher::Fetcher;

mod cache;
mod config;
mod console;
mod error;
mod fetcher;
mod git_utils;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    env_logger::init();

    let mut config = Config::new();
    let cli = Cli::parse();
    if cli.reload || config.get_hash(&cli.repo).is_none() {
        let _ = config.update_hash(&cli.repo);
    }

    let mut fetcher = Fetcher::new(&mut config, &cli.repo);
    fetcher.go(cli.project_name).await?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    repo: String,
    project_name: Option<String>,

    #[arg(short, long)]
    reload: bool,
}
