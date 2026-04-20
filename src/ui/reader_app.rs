use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use std::time::{Duration, Instant};

use crate::{
    reader::{anchor, page::Page, page, sanitize, styled},
    store::{bookmarks::{self, Bookmark}, db::Db},
    ui::{
        chrome::{Chrome, ChromeState},
        keymap::{Action, defaults, table::{Dispatch, Keymap}},
        reader_view::ReaderView,
        terminal::{self, Tui},
    },
};

#[derive(Debug, Clone, Copy)]
enum MarkMode { Set, Jump }

pub struct ReaderApp {
    pub pages: Vec<Page>,
    pub page_idx: usize,
    pub row_idx: usize,
    pub column_width: u16,
    pub theme: String,
    pub chrome: Chrome,
    pub title: String,
    pub keymap: Keymap,
    pub spine_idx: u32,
    pub book_id: Option<i64>,
    pub db: Option<Db>,
    pending_mark: Option<MarkMode>,
    plain_text: String,
}

pub fn run_with_html(html: &str, title: &str) -> Result<()> {
    run_with_html_and_db(html, title, None, None, 0)
}

pub fn run_with_html_and_db(
    html: &str,
    title: &str,
    db: Option<Db>,
    book_id: Option<i64>,
    spine_idx: u32,
) -> Result<()> {
    let safe = sanitize::clean(html);
    let spans = styled::to_spans(&safe);
    let plain_text: String = spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().concat();

    let mut term = terminal::enter()?;
    let size = term.size()?;
    let col = 68u16.min(size.width);
    let pages = page::paginate(&spans, col, size.height.saturating_sub(2));
    let keymap = Keymap::from_config(&defaults::default_entries())?;

    let mut app = ReaderApp {
        pages, page_idx: 0, row_idx: 0, column_width: col,
        theme: "dark".into(), chrome: Chrome::new(Duration::from_millis(3000)),
        title: title.to_string(), keymap,
        spine_idx, book_id, db,
        pending_mark: None, plain_text,
    };

    let res = event_loop(&mut term, &mut app);
    terminal::leave(&mut term)?;
    res
}

fn current_char_offset(app: &ReaderApp) -> u64 {
    app.pages.get(app.page_idx)
        .and_then(|p| p.rows.first())
        .map(|r| r.char_offset as u64)
        .unwrap_or(0)
}

fn event_loop(term: &mut Tui, app: &mut ReaderApp) -> Result<()> {
    loop {
        let now = Instant::now();
        term.draw(|f| {
            let area = f.size();
            let show_chrome = matches!(app.chrome.state(now), ChromeState::Visible);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(if show_chrome { 1 } else { 1 })])
                .split(area);
            ReaderView { page: app.pages.get(app.page_idx), column_width: app.column_width, theme: &app.theme }.render(f, chunks[0]);
            let status = format!(" {} · page {}/{} ", app.title, app.page_idx + 1, app.pages.len());
            f.render_widget(ratatui::widgets::Paragraph::new(status), chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                app.chrome.touch(Instant::now());

                // Handle pending mark follow-up letter first (before keymap).
                if let Some(mode) = app.pending_mark {
                    if let KeyCode::Char(letter) = k.code {
                        if letter.is_ascii_alphabetic() {
                            handle_mark(mode, letter, app)?;
                            app.pending_mark = None;
                            continue;
                        }
                    }
                    // Anything else cancels the pending mark.
                    app.pending_mark = None;
                    continue;
                }

                match app.keymap.feed(&key_to_raw(k)) {
                    Dispatch::Fire(Action::MoveDown) |
                    Dispatch::Fire(Action::PageDown) |
                    Dispatch::Fire(Action::HalfPageDown) => {
                        app.page_idx = (app.page_idx + 1).min(app.pages.len().saturating_sub(1));
                    }
                    Dispatch::Fire(Action::MoveUp) |
                    Dispatch::Fire(Action::PageUp) |
                    Dispatch::Fire(Action::HalfPageUp) => {
                        app.page_idx = app.page_idx.saturating_sub(1);
                    }
                    Dispatch::Fire(Action::GotoTop) => app.page_idx = 0,
                    Dispatch::Fire(Action::GotoBottom) => app.page_idx = app.pages.len().saturating_sub(1),
                    Dispatch::Fire(Action::QuitToLibrary) => break,
                    Dispatch::Fire(Action::ToggleTheme) => {
                        app.theme = match app.theme.as_str() {
                            "dark" => "sepia".into(),
                            "sepia" => "light".into(),
                            _ => "dark".into(),
                        };
                    }
                    Dispatch::Fire(Action::MarkSetPrompt)  => { app.pending_mark = Some(MarkMode::Set); }
                    Dispatch::Fire(Action::MarkJumpPrompt) => { app.pending_mark = Some(MarkMode::Jump); }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn handle_mark(mode: MarkMode, letter: char, app: &mut ReaderApp) -> Result<()> {
    let Some(db) = app.db.as_ref() else { return Ok(()); };
    let Some(book_id) = app.book_id else { return Ok(()); };
    let mark = letter.to_string();
    match mode {
        MarkMode::Set => {
            let co = current_char_offset(app);
            let bm = Bookmark {
                book_id,
                mark,
                spine_idx: app.spine_idx,
                char_offset: co,
                anchor_hash: anchor::anchor_hash(&app.plain_text, co as usize),
            };
            let mut conn = db.conn()?;
            bookmarks::set_bookmark(&mut conn, &bm)?;
        }
        MarkMode::Jump => {
            let conn = db.conn()?;
            if let Some(bm) = bookmarks::get_bookmark(&conn, book_id, &mark)? {
                if bm.spine_idx == app.spine_idx {
                    // Same spine — seek inside current paginated view.
                    let target = bm.char_offset as usize;
                    let best = app.pages.iter().position(|p|
                        p.rows.first().map(|r| r.char_offset).unwrap_or(0) >= target
                    ).unwrap_or(app.page_idx);
                    app.page_idx = best;
                }
                // Cross-spine jump is a no-op for v1; full support arrives with library integration.
            }
        }
    }
    Ok(())
}

fn key_to_raw(k: KeyEvent) -> String {
    match k.code {
        KeyCode::Char(c) if k.modifiers.contains(KeyModifiers::CONTROL) => format!("<C-{c}>"),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Up => "<Up>".into(),
        KeyCode::Down => "<Down>".into(),
        KeyCode::Enter => "<Enter>".into(),
        KeyCode::Esc => "<Esc>".into(),
        KeyCode::F(n) => format!("<F{n}>"),
        _ => String::new(),
    }
}
