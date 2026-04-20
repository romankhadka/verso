use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use rbook::Ebook;
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

fn loop_body(term: &mut Tui, db: &Db, _library_path: &std::path::Path,
             selected: &mut usize, sort: &mut Sort, filter: &mut Filter) -> Result<()> {
    loop {
        let rows: Vec<Row> = list_rows(&db.conn()?, *sort, *filter)?;
        if !rows.is_empty() { *selected = (*selected).min(rows.len() - 1); }

        term.draw(|f| LibraryView {
            rows: &rows, selected: *selected,
            sort_label: "last-read", filter_label: "all",
        }.render(f, f.size()))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => if *selected + 1 < rows.len() { *selected += 1 },
                    KeyCode::Char('k') | KeyCode::Up   => if *selected > 0 { *selected -= 1 },
                    KeyCode::Char('s') => *sort = cycle_sort(*sort),
                    KeyCode::Char('f') => *filter = cycle_filter(*filter),
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

fn cycle_sort(s: Sort) -> Sort {
    use Sort::*;
    match s { LastRead => Title, Title => Author, Author => Progress, Progress => Added, Added => LastRead }
}
fn cycle_filter(f: Filter) -> Filter {
    use Filter::*;
    match f { All => Reading, Reading => Unread, Unread => Finished, Finished => Broken, Broken => All }
}
