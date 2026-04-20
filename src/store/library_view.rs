use rusqlite::Connection;

#[derive(Debug, Clone, Copy)]
pub enum Sort { LastRead, Title, Author, Progress, Added }

#[derive(Debug, Clone, Copy)]
pub enum Filter { All, Reading, Unread, Finished, Broken }

#[derive(Debug, Clone)]
pub struct Row {
    pub book_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub pages: Option<u64>,
    pub progress_pct: Option<f32>,
    pub time_left_s: Option<u64>,
    pub last_read_at: Option<String>,
    pub finished_at: Option<String>,
    pub parse_error: Option<String>,
}

pub fn list_rows(c: &Connection, sort: Sort, filter: Filter) -> anyhow::Result<Vec<Row>> {
    let mut where_sql = "WHERE b.deleted_at IS NULL".to_string();
    match filter {
        Filter::Reading  => where_sql.push_str(" AND p.percent IS NOT NULL AND (b.finished_at IS NULL) AND p.percent > 0"),
        Filter::Unread   => where_sql.push_str(" AND (p.percent IS NULL OR p.percent = 0)"),
        Filter::Finished => where_sql.push_str(" AND b.finished_at IS NOT NULL"),
        Filter::Broken   => where_sql.push_str(" AND b.parse_error IS NOT NULL"),
        Filter::All      => {}
    }
    let order_sql = match sort {
        Sort::LastRead => "ORDER BY p.last_read_at DESC NULLS LAST",
        Sort::Title    => "ORDER BY b.title_norm ASC",
        Sort::Author   => "ORDER BY b.author_norm ASC",
        Sort::Progress => "ORDER BY p.percent DESC NULLS LAST",
        Sort::Added    => "ORDER BY b.added_at DESC",
    };
    let sql = format!("SELECT b.id, b.title, b.author, b.page_count,
                              p.percent, p.last_read_at, b.finished_at, b.parse_error,
                              b.word_count, p.words_read
                       FROM books b LEFT JOIN progress p ON p.book_id = b.id
                       {where_sql} {order_sql}");
    let mut stmt = c.prepare(&sql)?;
    let mut out = Vec::new();
    let rows = stmt.query_map([], |r| Ok({
        let pages: Option<u64> = r.get(3)?;
        let percent: Option<f32> = r.get(4)?;
        let word_count: Option<u64> = r.get(8)?;
        let words_read: Option<u64> = r.get(9)?;
        let time_left_s = match (word_count, percent) {
            (Some(w), Some(p)) => {
                let remaining_words = (w as f32 * (1.0 - p / 100.0)).max(0.0) as u64;
                Some((remaining_words as f64 / 250.0 * 60.0) as u64)
            }
            _ => None,
        };
        let _ = words_read;
        Row {
            book_id: r.get(0)?, title: r.get(1)?, author: r.get(2)?,
            pages, progress_pct: percent, time_left_s,
            last_read_at: r.get(5)?, finished_at: r.get(6)?, parse_error: r.get(7)?,
        }
    }))?;
    for row in rows { out.push(row?); }
    Ok(out)
}
