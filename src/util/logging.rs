use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialise tracing with daily-rotated file logs at `log_dir`.
/// Returns a guard that must live for the duration of the program.
pub fn init(log_dir: &Path) -> anyhow::Result<WorkerGuard> {
    std::fs::create_dir_all(log_dir)?;
    let appender = tracing_appender::rolling::daily(log_dir, "verso.log");
    let (nb, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_env("VERSO_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(nb).with_ansi(false))
        .init();

    Ok(guard)
}
