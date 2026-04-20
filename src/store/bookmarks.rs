use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub book_id:    i64,
    pub mark:       String,
    pub spine_idx:  u32,
    pub char_offset:u64,
    pub anchor_hash:String,
}

pub fn set_bookmark(c: &mut Connection, b: &Bookmark) -> anyhow::Result<()> {
    c.execute(
        "INSERT INTO bookmarks(book_id, mark, spine_idx, char_offset, anchor_hash)
         VALUES (?,?,?,?,?)
         ON CONFLICT(book_id, mark) DO UPDATE SET
           spine_idx=excluded.spine_idx,
           char_offset=excluded.char_offset,
           anchor_hash=excluded.anchor_hash,
           created_at=CURRENT_TIMESTAMP",
        params![b.book_id, b.mark, b.spine_idx, b.char_offset, b.anchor_hash],
    )?;
    Ok(())
}

pub fn get_bookmark(c: &Connection, book_id: i64, mark: &str) -> anyhow::Result<Option<Bookmark>> {
    Ok(c.query_row(
        "SELECT book_id, mark, spine_idx, char_offset, anchor_hash
         FROM bookmarks WHERE book_id = ? AND mark = ?",
        params![book_id, mark],
        |r| Ok(Bookmark {
            book_id: r.get(0)?, mark: r.get(1)?, spine_idx: r.get(2)?,
            char_offset: r.get(3)?, anchor_hash: r.get(4)?,
        }),
    ).optional()?)
}
