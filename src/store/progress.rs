use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub struct ProgressRow {
    pub book_id: i64,
    pub spine_idx: u32,
    pub char_offset: u64,
    pub anchor_hash: String,
    pub percent: f32,
    pub time_read_s: u64,
    pub words_read: u64,
}

/// Insert or update the progress row for a book. `last_read_at` is always
/// refreshed to `CURRENT_TIMESTAMP`.
pub fn upsert(c: &mut Connection, row: &ProgressRow) -> anyhow::Result<()> {
    c.execute(
        "INSERT INTO progress(book_id, spine_idx, char_offset, anchor_hash,
                              percent, time_read_s, words_read, last_read_at)
         VALUES (?,?,?,?,?,?,?, CURRENT_TIMESTAMP)
         ON CONFLICT(book_id) DO UPDATE SET
           spine_idx    = excluded.spine_idx,
           char_offset  = excluded.char_offset,
           anchor_hash  = excluded.anchor_hash,
           percent      = excluded.percent,
           time_read_s  = excluded.time_read_s,
           words_read   = excluded.words_read,
           last_read_at = CURRENT_TIMESTAMP",
        params![
            row.book_id,
            row.spine_idx,
            row.char_offset,
            row.anchor_hash,
            row.percent,
            row.time_read_s,
            row.words_read,
        ],
    )?;
    Ok(())
}

/// Load the progress row for a book. Returns `Ok(None)` when no row exists.
pub fn load(c: &Connection, book_id: i64) -> anyhow::Result<Option<ProgressRow>> {
    Ok(c.query_row(
        "SELECT book_id, spine_idx, char_offset, anchor_hash,
                percent, time_read_s, words_read
         FROM progress WHERE book_id = ?",
        params![book_id],
        |r| {
            Ok(ProgressRow {
                book_id: r.get(0)?,
                spine_idx: r.get(1)?,
                char_offset: r.get(2)?,
                anchor_hash: r.get(3)?,
                percent: r.get(4)?,
                time_read_s: r.get(5)?,
                words_read: r.get(6)?,
            })
        },
    )
    .optional()?)
}
