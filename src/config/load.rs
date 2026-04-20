use super::Config;
use std::path::Path;

pub fn from_path(path: &Path) -> anyhow::Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(path)?;
    let cfg: Config = toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", path.display(), e))?;
    Ok(cfg)
}

/// Deserialise from a TOML string. Used by tests.
pub fn from_str(s: &str) -> anyhow::Result<Config> {
    Ok(toml::from_str(s)?)
}
