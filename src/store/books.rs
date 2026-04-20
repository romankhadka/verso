use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub struct BookRow {
    pub stable_id: Option<String>,
    pub file_hash: Option<String>,
    pub title_norm: String,
    pub author_norm: Option<String>,
    pub path: String,
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_at: Option<String>,
    pub word_count: Option<u64>,
    pub page_count: Option<u64>,
    pub parse_error: Option<String>,
}

impl BookRow {
    /// Build a fixture row keyed by a short name. For tests only.
    pub fn new_fixture(name: &str) -> Self {
        Self {
            stable_id: Some(format!("urn:fixture:{name}")),
            file_hash: Some(format!("{name}-hash")),
            title_norm: format!("fixture {name}"),
            author_norm: Some("fixture author".into()),
            path: format!("/tmp/{name}.epub"),
            title: format!("Fixture {name}"),
            author: Some("Fixture Author".into()),
            language: Some("en".into()),
            publisher: None,
            published_at: None,
            word_count: Some(1000),
            page_count: Some(4),
            parse_error: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum IdentityMatch {
    ById(i64),
    ByHash(i64),
    ByNorm(i64),
}

pub fn resolve_identity(c: &Connection, row: &BookRow) -> anyhow::Result<Option<IdentityMatch>> {
    if let Some(sid) = &row.stable_id {
        if let Some(id) = c
            .query_row(
                "SELECT id FROM books WHERE stable_id = ? AND deleted_at IS NULL",
                params![sid],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        {
            return Ok(Some(IdentityMatch::ById(id)));
        }
    }
    if let Some(fh) = &row.file_hash {
        if let Some(id) = c
            .query_row(
                "SELECT id FROM books WHERE file_hash = ? AND deleted_at IS NULL",
                params![fh],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        {
            return Ok(Some(IdentityMatch::ByHash(id)));
        }
    }
    if let Some(a) = &row.author_norm {
        if let Some(id) = c.query_row(
            "SELECT id FROM books WHERE title_norm = ? AND author_norm = ? AND deleted_at IS NULL",
            params![row.title_norm, a], |r| r.get::<_, i64>(0),
        ).optional()? { return Ok(Some(IdentityMatch::ByNorm(id))); }
    }
    Ok(None)
}

/// Upsert a book row. Returns the row id.
pub fn upsert(c: &mut Connection, row: &BookRow) -> anyhow::Result<i64> {
    let tx = c.transaction()?;
    let existing = resolve_identity(&tx, row)?;
    let id = match existing {
        Some(IdentityMatch::ById(id) | IdentityMatch::ByHash(id) | IdentityMatch::ByNorm(id)) => {
            tx.execute(
                "UPDATE books SET stable_id = COALESCE(?, stable_id),
                                   file_hash = COALESCE(?, file_hash),
                                   title_norm = ?, author_norm = ?,
                                   path = ?, title = ?, author = ?, language = ?,
                                   publisher = ?, published_at = ?,
                                   word_count = ?, page_count = ?, parse_error = ?,
                                   deleted_at = NULL
                 WHERE id = ?",
                params![
                    row.stable_id,
                    row.file_hash,
                    row.title_norm,
                    row.author_norm,
                    row.path,
                    row.title,
                    row.author,
                    row.language,
                    row.publisher,
                    row.published_at,
                    row.word_count,
                    row.page_count,
                    row.parse_error,
                    id
                ],
            )?;
            id
        }
        None => {
            tx.execute(
                "INSERT INTO books (stable_id, file_hash, title_norm, author_norm,
                                    path, title, author, language, publisher, published_at,
                                    word_count, page_count, parse_error)
                 VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
                params![
                    row.stable_id,
                    row.file_hash,
                    row.title_norm,
                    row.author_norm,
                    row.path,
                    row.title,
                    row.author,
                    row.language,
                    row.publisher,
                    row.published_at,
                    row.word_count,
                    row.page_count,
                    row.parse_error
                ],
            )?;
            tx.last_insert_rowid()
        }
    };
    tx.commit()?;
    Ok(id)
}
