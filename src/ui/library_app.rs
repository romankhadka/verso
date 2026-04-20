use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use rbook::Ebook;
use ratatui::layout::Rect;
use std::time::Duration;
use crate::{
    store::{db::Db, library_view::{list_rows, Sort, Filter, Row}},
    ui::{library_view::LibraryView, reader_app, terminal::{self, Tui}},
};

pub fn run(db: &Db, library_path: &std::path::Path) -> Result<()> {
    let mut term = terminal::enter()?;
    let mut selected = 0usize;
    let mut sort = Sort::LastRead;
    let mut filter = Filter::All;

    let res = loop_body(&mut term, db, library_path, &mut selected, &mut sort, &mut filter);
    terminal::leave(&mut term)?;
    res
}

struct Details {
    path: String,
    added_at: String,
    finished_at: Option<String>,
    parse_error: Option<String>,
    highlights_count: i64,
    bookmarks_count: i64,
}

fn fetch_details(db: &Db, book_id: i64) -> Result<Details> {
    let conn = db.conn()?;
    let (path, added_at, finished_at, parse_error): (String, String, Option<String>, Option<String>) =
        conn.query_row(
            "SELECT path, added_at, finished_at, parse_error FROM books WHERE id = ?",
            rusqlite::params![book_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
    let (highlights_count, bookmarks_count): (i64, i64) = conn.query_row(
        "SELECT (SELECT COUNT(*) FROM highlights WHERE book_id = ?),
                (SELECT COUNT(*) FROM bookmarks  WHERE book_id = ?)",
        rusqlite::params![book_id, book_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    Ok(Details { path, added_at, finished_at, parse_error, highlights_count, bookmarks_count })
}

fn build_details_text(row: &Row, d: &Details) -> String {
    let mut lines = Vec::<String>::new();
    lines.push(format!("Title:       {}", row.title));
    lines.push(format!("Author:      {}", row.author.clone().unwrap_or_else(|| "—".into())));
    lines.push(format!("Path:        {}", d.path));
    lines.push(format!("Added:       {}", d.added_at));
    lines.push(format!("Finished:    {}", d.finished_at.clone().unwrap_or_else(|| "—".into())));
    lines.push(format!("Highlights:  {}", d.highlights_count));
    lines.push(format!("Bookmarks:   {}", d.bookmarks_count));
    if let Some(e) = &d.parse_error {
        lines.push(String::new());
        lines.push(format!("Parse error: {e}"));
    }
    lines.push(String::new());
    lines.push("[d / Esc to close]".into());
    lines.join("\n")
}

fn loop_body(term: &mut Tui, db: &Db, _library_path: &std::path::Path,
             selected: &mut usize, sort: &mut Sort, filter: &mut Filter) -> Result<()> {
    let mut details_open = false;
    loop {
        let rows: Vec<Row> = list_rows(&db.conn()?, *sort, *filter)?;
        if !rows.is_empty() { *selected = (*selected).min(rows.len() - 1); }

        let details: Option<Details> = if details_open {
            rows.get(*selected).and_then(|r| fetch_details(db, r.book_id).ok())
        } else {
            None
        };

        term.draw(|f| {
            let area = f.size();
            LibraryView {
                rows: &rows, selected: *selected,
                sort_label: "last-read", filter_label: "all",
            }.render(f, area);

            if let (true, Some(row), Some(d)) = (details_open, rows.get(*selected), details.as_ref()) {
                let panel = Rect {
                    x: area.x + area.width / 5,
                    y: area.y + area.height / 5,
                    width: (area.width * 3 / 5).max(40),
                    height: (area.height * 3 / 5).max(10),
                };
                let details_text = build_details_text(row, d);
                f.render_widget(ratatui::widgets::Clear, panel);
                let block = ratatui::widgets::Block::default()
                    .title(" Details ")
                    .borders(ratatui::widgets::Borders::ALL);
                let para = ratatui::widgets::Paragraph::new(details_text).block(block);
                f.render_widget(para, panel);
            }
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                if details_open {
                    match k.code {
                        KeyCode::Char('d') | KeyCode::Esc => details_open = false,
                        KeyCode::Char('j') | KeyCode::Down => if *selected + 1 < rows.len() { *selected += 1 },
                        KeyCode::Char('k') | KeyCode::Up   => if *selected > 0 { *selected -= 1 },
                        _ => {}
                    }
                } else {
                    match k.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => if *selected + 1 < rows.len() { *selected += 1 },
                        KeyCode::Char('k') | KeyCode::Up   => if *selected > 0 { *selected -= 1 },
                        KeyCode::Char('s') => *sort = cycle_sort(*sort),
                        KeyCode::Char('f') => *filter = cycle_filter(*filter),
                        KeyCode::Char('d') => details_open = true,
                        KeyCode::Esc => {}
                        KeyCode::Enter => {
                            if let Some(row) = rows.get(*selected) {
                                let path: String = db.conn()?.query_row(
                                    "SELECT path FROM books WHERE id = ?",
                                    rusqlite::params![row.book_id],
                                    |r| r.get(0),
                                )?;
                                terminal::leave(term)?;
                                let book = rbook::Epub::new(std::path::Path::new(&path))?;
                                let spine = book.spine().elements();
                                let first = spine.first().ok_or_else(|| anyhow::anyhow!("empty spine"))?;
                                let idref = first.name();
                                let manifest_item = book.manifest().by_id(idref)
                                    .ok_or_else(|| anyhow::anyhow!("manifest missing idref {}", idref))?;
                                let html = book.read_file(manifest_item.value())?;
                                let reader_db = Db::open(db.location())?;
                                reader_app::run_with_html_and_db(&html, &row.title, Some(reader_db), Some(row.book_id), 0)?;
                                *term = terminal::enter()?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn cycle_sort(s: Sort) -> Sort {
    use Sort::*;
    match s { LastRead => Title, Title => Author, Author => Progress, Progress => Added, Added => LastRead }
}
fn cycle_filter(f: Filter) -> Filter {
    use Filter::*;
    match f { All => Reading, Reading => Unread, Unread => Finished, Finished => Broken, Broken => All }
}
