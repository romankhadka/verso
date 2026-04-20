use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

/// Owns the SQLite path and produces fresh connections.
/// A "connection" here is a short-lived handle; the store-writer thread owns a long-lived one.
pub struct Db {
    path: PathBuf,
}

impl Db {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
        // Validate we can open.
        let _ = Self::new_conn(path)?;
        Ok(Self { path: path.to_path_buf() })
    }

    pub fn conn(&self) -> anyhow::Result<Connection> {
        Self::new_conn(&self.path)
    }

    fn new_conn(path: &Path) -> anyhow::Result<Connection> {
        let c = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        c.pragma_update(None, "journal_mode", "WAL")?;
        c.pragma_update(None, "synchronous", "NORMAL")?;
        c.pragma_update(None, "busy_timeout", 5000i64)?;
        c.pragma_update(None, "foreign_keys", "ON")?;
        Ok(c)
    }

    pub fn migrate(&self) -> anyhow::Result<()> {
        super::migrate::run(&mut self.conn()?)
    }

    pub fn location(&self) -> &std::path::Path { &self.path }
}
