use crate::{
    library::{epub_guard, epub_meta, hashing, normalise},
    store::{books::{upsert, BookRow}, db::Db},
};
use std::path::Path;

#[derive(Debug, Default)]
pub struct ScanReport {
    pub inserted: usize,
    pub updated:  usize,
    pub skipped:  usize,
    pub errors:   Vec<(std::path::PathBuf, String)>,
}

pub fn scan_folder(dir: &Path, db: &Db) -> anyhow::Result<ScanReport> {
    let mut report = ScanReport::default();
    let mut conn = db.conn()?;
    for entry in walkdir(dir) {
        let path = entry;
        if path.extension().and_then(|s| s.to_str()) != Some("epub") { continue; }

        if let Err(e) = epub_guard::validate_archive(&path, epub_guard::Limits::default()) {
            report.errors.push((path.clone(), e.to_string()));
            continue;
        }

        let meta = match epub_meta::extract(&path) {
            Ok(m) => m,
            Err(e) => { report.errors.push((path.clone(), e.to_string())); continue; }
        };

        let file_hash = hashing::sha256_file(&path).ok();
        let row = BookRow {
            stable_id:    meta.stable_id.clone(),
            file_hash,
            title_norm:   normalise::normalise_text(&meta.title),
            author_norm:  meta.author.as_deref().map(normalise::normalise_author),
            path:         path.to_string_lossy().to_string(),
            title:        meta.title,
            author:       meta.author,
            language:     meta.language,
            publisher:    meta.publisher,
            published_at: meta.published_at,
            word_count:   meta.word_count,
            page_count:   meta.word_count.map(|w| (w / 275).max(1)),
            parse_error:  None,
        };
        match upsert(&mut conn, &row)? {
            _id => { report.inserted += 1; } // For v1 we just count all as "inserted"; refine later.
        }
    }

    // Soft-delete books whose on-disk file has vanished (only books under this scan dir).
    let dir_prefix = dir.to_string_lossy().to_string();
    let orphaned: Vec<(i64, String)> = conn.prepare(
        "SELECT id, path FROM books WHERE deleted_at IS NULL AND path LIKE ? || '%'"
    )?
    .query_map(rusqlite::params![dir_prefix], |r| Ok((r.get(0)?, r.get(1)?)))?
    .collect::<Result<_, _>>()?;

    for (id, p) in orphaned {
        if !std::path::Path::new(&p).exists() {
            conn.execute(
                "UPDATE books SET deleted_at = CURRENT_TIMESTAMP WHERE id = ?",
                rusqlite::params![id],
            )?;
        }
    }

    Ok(report)
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { out.extend(walkdir(&p)); }
            else { out.push(p); }
        }
    }
    out
}
