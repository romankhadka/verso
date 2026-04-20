use anyhow::Result;
use clap::Parser;
use verso::{cli::Cli, config::load as config_load, util::{logging, paths::Paths}};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::from_env()?;
    let _guard = logging::init(&paths.log_dir())?;
    let _cfg = config_load::from_path(&paths.config_file())?;
    tracing::info!("verso {} starting (cmd={:?})", env!("CARGO_PKG_VERSION"), cli.command);
    Ok(())
}
