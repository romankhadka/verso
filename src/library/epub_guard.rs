use std::path::Path;
use thiserror::Error;

pub struct Limits {
    pub max_decompressed_bytes: u64,
    pub max_entry_bytes:        u64,
    pub max_entries:            usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self { max_decompressed_bytes: 256 * 1024 * 1024, max_entry_bytes: 16 * 1024 * 1024, max_entries: 10_000 }
    }
}

#[derive(Debug, Error)]
pub enum GuardError {
    #[error("path traversal attempt in zip entry: {0}")]
    PathTraversal(String),
    #[error("archive entry count {0} exceeds limit {1}")]
    TooManyEntries(usize, usize),
    #[error("entry {0} size {1} exceeds per-entry limit {2}")]
    EntryTooLarge(String, u64, u64),
    #[error("total decompressed size {0} exceeds limit {1}")]
    TotalTooLarge(u64, u64),
    #[error("symlink entry not allowed: {0}")]
    Symlink(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
}

/// Validate an EPUB's ZIP structure without extracting to disk.
pub fn validate_archive(path: &Path, limits: Limits) -> Result<(), GuardError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if archive.len() > limits.max_entries {
        return Err(GuardError::TooManyEntries(archive.len(), limits.max_entries));
    }

    let mut total: u64 = 0;
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
            return Err(GuardError::PathTraversal(name));
        }
        if is_symlink(&entry) {
            return Err(GuardError::Symlink(name));
        }
        if entry.size() > limits.max_entry_bytes {
            return Err(GuardError::EntryTooLarge(name, entry.size(), limits.max_entry_bytes));
        }
        total = total.saturating_add(entry.size());
        if total > limits.max_decompressed_bytes {
            return Err(GuardError::TotalTooLarge(total, limits.max_decompressed_bytes));
        }
    }
    Ok(())
}

fn is_symlink(entry: &zip::read::ZipFile<'_>) -> bool {
    // zip 0.6 does not expose `is_symlink`; detect via unix mode bits.
    // S_IFLNK = 0o120000.
    entry.unix_mode().map(|m| (m & 0o170000) == 0o120000).unwrap_or(false)
}
