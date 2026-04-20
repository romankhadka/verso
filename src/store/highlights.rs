use rusqlite::{params, Connection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorStatus { Ok, Drifted, Lost }

impl AnchorStatus {
    pub fn as_str(self) -> &'static str {
        match self { Self::Ok => "ok", Self::Drifted => "drifted", Self::Lost => "lost" }
    }
    pub fn parse(s: &str) -> Self {
        match s { "drifted" => Self::Drifted, "lost" => Self::Lost, _ => Self::Ok }
    }
}

#[derive(Debug, Clone)]
pub struct Highlight {
    pub id: i64,
    pub book_id: i64,
    pub spine_idx: u32,
    pub chapter_title: Option<String>,
    pub char_offset_start: u64,
    pub char_offset_end: u64,
    pub text: String,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
    pub note: Option<String>,
    pub anchor_status: AnchorStatus,
}

pub fn insert(c: &mut Connection, h: &Highlight) -> anyhow::Result<i64> {
    c.execute(
        "INSERT INTO highlights(book_id, spine_idx, chapter_title, char_offset_start, char_offset_end,
                                text, context_before, context_after, note, anchor_status)
         VALUES (?,?,?,?,?,?,?,?,?,?)",
        params![h.book_id, h.spine_idx, h.chapter_title, h.char_offset_start, h.char_offset_end,
                h.text, h.context_before, h.context_after, h.note, h.anchor_status.as_str()],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn list(c: &Connection, book_id: i64) -> anyhow::Result<Vec<Highlight>> {
    let mut stmt = c.prepare(
        "SELECT id, book_id, spine_idx, chapter_title, char_offset_start, char_offset_end,
                text, context_before, context_after, note, anchor_status
         FROM highlights WHERE book_id = ? ORDER BY spine_idx, char_offset_start",
    )?;
    let rows: Vec<Highlight> = stmt.query_map(params![book_id], |r| Ok(Highlight {
        id: r.get(0)?, book_id: r.get(1)?, spine_idx: r.get(2)?, chapter_title: r.get(3)?,
        char_offset_start: r.get(4)?, char_offset_end: r.get(5)?, text: r.get(6)?,
        context_before: r.get(7)?, context_after: r.get(8)?, note: r.get(9)?,
        anchor_status: AnchorStatus::parse(&r.get::<_, String>(10)?),
    }))?.collect::<Result<_,_>>()?;
    Ok(rows)
}
