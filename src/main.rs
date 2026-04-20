use anyhow::Result;
use verso::util::{logging, paths::Paths};

fn main() -> Result<()> {
    let paths = Paths::from_env()?;
    let _guard = logging::init(&paths.log_dir())?;
    tracing::info!("verso {} starting", env!("CARGO_PKG_VERSION"));
    Ok(())
}
