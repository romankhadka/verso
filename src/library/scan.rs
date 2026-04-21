use crate::{
    library::{epub_guard, epub_meta, hashing, normalise, reanchor},
    store::{
        books::{resolve_identity, upsert, BookRow, IdentityMatch},
        db::Db,
    },
};
use std::path::Path;

#[derive(Debug, Default)]
pub struct ScanReport {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: Vec<(std::path::PathBuf, String)>,
}

pub fn scan_folder(dir: &Path, db: &Db) -> anyhow::Result<ScanReport> {
    let mut report = ScanReport::default();
    let mut conn = db.conn()?;
    for entry in walkdir(dir) {
        let path = entry;
        if path.extension().and_then(|s| s.to_str()) != Some("epub") {
            continue;
        }

        if let Err(e) = epub_guard::validate_archive(&path, epub_guard::Limits::default()) {
            let err_string = e.to_string();
            record_broken(&mut conn, &path, &err_string);
            report.errors.push((path.clone(), err_string));
            continue;
        }

        let meta = match epub_meta::extract(&path) {
            Ok(m) => m,
            Err(e) => {
                let err_string = e.to_string();
                record_broken(&mut conn, &path, &err_string);
                report.errors.push((path.clone(), err_string));
                continue;
            }
        };

        let file_hash = hashing::sha256_file(&path).ok();
        let row = BookRow {
            stable_id: meta.stable_id.clone(),
            file_hash,
            title_norm: normalise::normalise_text(&meta.title),
            author_norm: meta.author.as_deref().map(normalise::normalise_author),
            path: path.to_string_lossy().to_string(),
            title: meta.title,
            author: meta.author,
            language: meta.language,
            publisher: meta.publisher,
            published_at: meta.published_at,
            word_count: meta.word_count,
            page_count: meta.word_count.map(|w| (w / 275).max(1)),
            parse_error: None,
        };
        // Detect pre-upsert hash state so we can trigger a highlight reanchor if
        // a re-imported edition's bytes have changed under an existing identity.
        let pre_hash: Option<String> = match resolve_identity(&conn, &row)? {
            Some(
                IdentityMatch::ById(id) | IdentityMatch::ByHash(id) | IdentityMatch::ByNorm(id),
            ) => conn
                .query_row(
                    "SELECT file_hash FROM books WHERE id = ?",
                    rusqlite::params![id],
                    |r| r.get::<_, Option<String>>(0),
                )
                .ok()
                .flatten(),
            None => None,
        };

        // For v1 we just count all as "inserted"; refine later.
        let book_id = upsert(&mut conn, &row)?;

        // If the row existed previously with a different hash, re-run anchor
        // resolution so highlights don't silently drift/go lost.
        if let (Some(pre), Some(post)) = (pre_hash, row.file_hash.as_ref()) {
            if pre != *post {
                let _ = reanchor::reanchor_book(db, book_id, &path);
            }
        }

        report.inserted += 1;
    }

    // Soft-delete books whose on-disk file has vanished (only books under this scan dir).
    let dir_prefix = dir.to_string_lossy().to_string();
    let orphaned: Vec<(i64, String)> = conn
        .prepare("SELECT id, path FROM books WHERE deleted_at IS NULL AND path LIKE ? || '%'")?
        .query_map(rusqlite::params![dir_prefix], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?
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

/// Upsert a minimal row marking this file as unparseable so it appears under
/// the "broken" library filter. Best-effort: a DB error here must not tank the
/// whole scan, so we discard the result.
fn record_broken(conn: &mut rusqlite::Connection, path: &Path, err_string: &str) {
    let title = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.epub")
        .to_string();
    let row = BookRow {
        stable_id: None,
        file_hash: hashing::sha256_file(path).ok(),
        title_norm: normalise::normalise_text(&title),
        author_norm: None,
        path: path.to_string_lossy().to_string(),
        title,
        author: None,
        language: None,
        publisher: None,
        published_at: None,
        word_count: None,
        page_count: None,
        parse_error: Some(err_string.to_string()),
    };
    let _ = upsert(conn, &row);
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}
