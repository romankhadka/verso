use std::path::{Path, PathBuf};

pub struct Paths {
    root: PathBuf,
}

impl Paths {
    /// Deterministic layout rooted at `root` (useful in tests).
    pub fn for_root(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// Real XDG layout derived from the user's environment.
    pub fn from_env() -> anyhow::Result<Self> {
        let dirs = directories::ProjectDirs::from("app", "", "verso")
            .ok_or_else(|| anyhow::anyhow!("could not determine XDG paths"))?;
        // We compose our own layout so data_dir/state_dir/config_dir each end in "verso".
        let root = dirs
            .data_dir()
            .parent()
            .ok_or_else(|| anyhow::anyhow!("unexpected XDG root"))?
            .parent()
            .unwrap_or(dirs.data_dir())
            .to_path_buf();
        Ok(Self { root })
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join("share").join("verso")
    }
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config").join("verso")
    }
    pub fn state_dir(&self) -> PathBuf {
        self.root.join("state").join("verso")
    }

    pub fn db_file(&self) -> PathBuf {
        self.data_dir().join("verso.db")
    }
    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }
    pub fn log_dir(&self) -> PathBuf {
        self.state_dir().join("log")
    }
}
