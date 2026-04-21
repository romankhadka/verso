use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use std::time::{Duration, Instant};

use crate::{
    reader::{
        anchor, page,
        page::Page,
        sanitize,
        search::{self, SearchDirection},
        styled,
    },
    store::{
        bookmarks::{self, Bookmark},
        db::Db,
        progress::{self, ProgressRow},
    },
    ui::{
        chrome::{Chrome, ChromeState},
        keymap::{
            defaults,
            table::{Dispatch, Keymap},
            Action,
        },
        reader_view::ReaderView,
        terminal::{self, Tui},
    },
};

#[derive(Debug, Clone, Copy)]
enum MarkMode {
    Set,
    Jump,
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Normal,
    Visual { anchor_char_offset: usize },
}

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
    pub mode: Mode,
    pending_mark: Option<MarkMode>,
    plain_text: String,
    plain_text_chars: usize,
    last_persist: Instant,
    search_buffer: String,
    search_mode: Option<SearchDirection>,
    search_matches: Vec<usize>,
    search_cursor: usize,
}

const PROGRESS_PERSIST_INTERVAL: Duration = Duration::from_secs(5);

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
    let plain_text: String = spans
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .concat();

    let mut term = terminal::enter()?;
    let size = term.size()?;
    let col = 68u16.min(size.width);
    let pages = page::paginate(&spans, col, size.height.saturating_sub(2));
    let keymap = Keymap::from_config(&defaults::default_entries())?;

    let plain_text_chars = plain_text.chars().count();

    let mut app = ReaderApp {
        pages,
        page_idx: 0,
        row_idx: 0,
        column_width: col,
        theme: "dark".into(),
        chrome: Chrome::new(Duration::from_millis(3000)),
        title: title.to_string(),
        keymap,
        spine_idx,
        book_id,
        db,
        mode: Mode::Normal,
        pending_mark: None,
        plain_text,
        plain_text_chars,
        last_persist: Instant::now(),
        search_buffer: String::new(),
        search_mode: None,
        search_matches: Vec::new(),
        search_cursor: 0,
    };

    restore_progress(&mut app);

    let res = event_loop(&mut term, &mut app);
    terminal::leave(&mut term)?;
    res
}

/// Restore `page_idx` from the persisted progress row, if one exists.
/// Finds the page whose first row's `char_offset` is the closest value
/// not exceeding `row.char_offset`.
fn restore_progress(app: &mut ReaderApp) {
    let (Some(db), Some(book_id)) = (app.db.as_ref(), app.book_id) else {
        return;
    };
    let Ok(conn) = db.conn() else {
        return;
    };
    let Ok(Some(row)) = progress::load(&conn, book_id) else {
        return;
    };
    let target = row.char_offset as usize;
    let mut best: Option<usize> = None;
    for (idx, page) in app.pages.iter().enumerate() {
        let Some(first) = page.rows.first() else {
            continue;
        };
        if first.char_offset <= target {
            best = Some(idx);
        } else {
            break;
        }
    }
    if let Some(idx) = best {
        app.page_idx = idx;
    }
}

fn save_progress(app: &mut ReaderApp) {
    let (Some(db), Some(book_id)) = (app.db.as_ref(), app.book_id) else {
        return;
    };
    let Ok(mut conn) = db.conn() else {
        return;
    };
    let char_offset = current_char_offset(app);
    let percent = if app.plain_text_chars == 0 {
        0.0
    } else {
        (char_offset as f32 / app.plain_text_chars as f32 * 100.0).clamp(0.0, 100.0)
    };
    let anchor_hash = anchor::anchor_hash(&app.plain_text, char_offset as usize);
    let row = ProgressRow {
        book_id,
        spine_idx: app.spine_idx,
        char_offset,
        anchor_hash,
        percent,
        time_read_s: 0,
        words_read: 0,
    };
    let _ = progress::upsert(&mut conn, &row);
    app.last_persist = Instant::now();
}

fn save_auto_bookmark(app: &mut ReaderApp) -> anyhow::Result<()> {
    let Some(db) = app.db.as_ref() else {
        return Ok(());
    };
    let Some(book_id) = app.book_id else {
        return Ok(());
    };
    let co = current_char_offset(app);
    let bm = Bookmark {
        book_id,
        mark: "\"".into(),
        spine_idx: app.spine_idx,
        char_offset: co,
        anchor_hash: anchor::anchor_hash(&app.plain_text, co as usize),
    };
    let mut conn = db.conn()?;
    bookmarks::set_bookmark(&mut conn, &bm)?;
    Ok(())
}

fn current_char_offset(app: &ReaderApp) -> u64 {
    app.pages
        .get(app.page_idx)
        .and_then(|p| p.rows.first())
        .map(|r| r.char_offset as u64)
        .unwrap_or(0)
}

fn seek_to_offset(app: &mut ReaderApp, target: usize) {
    let best = app
        .pages
        .iter()
        .position(|p| p.rows.iter().any(|r| r.char_offset >= target))
        .unwrap_or(app.page_idx);
    app.page_idx = best;
}

fn event_loop(term: &mut Tui, app: &mut ReaderApp) -> Result<()> {
    loop {
        let now = Instant::now();
        if now.duration_since(app.last_persist) >= PROGRESS_PERSIST_INTERVAL {
            save_progress(app);
        }
        term.draw(|f| {
            let area = f.size();
            let _show_chrome = matches!(app.chrome.state(now), ChromeState::Visible);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(area);
            ReaderView {
                page: app.pages.get(app.page_idx),
                column_width: app.column_width,
                theme: &app.theme,
            }
            .render(f, chunks[0]);
            if app.search_mode.is_some() {
                let prefix = match app.search_mode {
                    Some(SearchDirection::Backward) => "?",
                    _ => "/",
                };
                let status = format!("{prefix}{}", app.search_buffer);
                f.render_widget(ratatui::widgets::Paragraph::new(status), chunks[1]);
            } else {
                let mode_str = match app.mode {
                    Mode::Visual { .. } => " [VIS] ",
                    Mode::Normal => "",
                };
                let status = format!(
                    "{} {} · page {}/{} ",
                    mode_str,
                    app.title,
                    app.page_idx + 1,
                    app.pages.len()
                );
                f.render_widget(ratatui::widgets::Paragraph::new(status), chunks[1]);
            }
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

                // Intercept all keys while in search-query entry mode.
                if let Some(dir) = app.search_mode {
                    match k.code {
                        KeyCode::Char(c) => {
                            app.search_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_buffer.pop();
                        }
                        KeyCode::Enter => {
                            app.search_matches =
                                search::find_matches(&app.plain_text, &app.search_buffer, dir);
                            if !app.search_matches.is_empty() {
                                let cur = current_char_offset(app) as usize;
                                let idx = match dir {
                                    SearchDirection::Forward => app
                                        .search_matches
                                        .iter()
                                        .position(|&m| m >= cur)
                                        .unwrap_or(0),
                                    SearchDirection::Backward => app
                                        .search_matches
                                        .iter()
                                        .rposition(|&m| m <= cur)
                                        .unwrap_or(app.search_matches.len() - 1),
                                };
                                app.search_cursor = idx;
                                let target = app.search_matches[idx];
                                seek_to_offset(app, target);
                            }
                            app.search_mode = None;
                        }
                        KeyCode::Esc => {
                            app.search_mode = None;
                        }
                        _ => {}
                    }
                    continue;
                }

                match app.keymap.feed(&key_to_raw(k)) {
                    Dispatch::Fire(Action::MoveDown)
                    | Dispatch::Fire(Action::PageDown)
                    | Dispatch::Fire(Action::HalfPageDown) => {
                        app.page_idx = (app.page_idx + 1).min(app.pages.len().saturating_sub(1));
                    }
                    Dispatch::Fire(Action::MoveUp)
                    | Dispatch::Fire(Action::PageUp)
                    | Dispatch::Fire(Action::HalfPageUp) => {
                        app.page_idx = app.page_idx.saturating_sub(1);
                    }
                    Dispatch::Fire(Action::GotoTop) => app.page_idx = 0,
                    Dispatch::Fire(Action::GotoBottom) => {
                        app.page_idx = app.pages.len().saturating_sub(1)
                    }
                    Dispatch::Fire(Action::QuitToLibrary) => match app.mode {
                        Mode::Visual { .. } => app.mode = Mode::Normal,
                        Mode::Normal => {
                            save_progress(app);
                            save_auto_bookmark(app)?;
                            break;
                        }
                    },
                    Dispatch::Fire(Action::VisualSelect) => {
                        let off = current_char_offset(app) as usize;
                        app.mode = Mode::Visual {
                            anchor_char_offset: off,
                        };
                    }
                    Dispatch::Fire(Action::YankHighlight) => {
                        if let Mode::Visual { anchor_char_offset } = app.mode {
                            let cur = current_char_offset(app) as usize;
                            let (start, end) = if cur >= anchor_char_offset {
                                (anchor_char_offset, cur)
                            } else {
                                (cur, anchor_char_offset)
                            };
                            save_highlight(app, start, end)?;
                            app.mode = Mode::Normal;
                        }
                    }
                    Dispatch::Fire(Action::ToggleTheme) => {
                        app.theme = match app.theme.as_str() {
                            "dark" => "sepia".into(),
                            "sepia" => "light".into(),
                            _ => "dark".into(),
                        };
                    }
                    Dispatch::Fire(Action::MarkSetPrompt) => {
                        app.pending_mark = Some(MarkMode::Set);
                    }
                    Dispatch::Fire(Action::MarkJumpPrompt) => {
                        app.pending_mark = Some(MarkMode::Jump);
                    }
                    Dispatch::Fire(Action::BeginSearchFwd) => {
                        app.search_mode = Some(SearchDirection::Forward);
                        app.search_buffer.clear();
                    }
                    Dispatch::Fire(Action::BeginSearchBack) => {
                        app.search_mode = Some(SearchDirection::Backward);
                        app.search_buffer.clear();
                    }
                    Dispatch::Fire(Action::SearchNext) if !app.search_matches.is_empty() => {
                        app.search_cursor = (app.search_cursor + 1) % app.search_matches.len();
                        let target = app.search_matches[app.search_cursor];
                        seek_to_offset(app, target);
                    }
                    Dispatch::Fire(Action::SearchPrev) if !app.search_matches.is_empty() => {
                        let len = app.search_matches.len();
                        app.search_cursor = (app.search_cursor + len - 1) % len;
                        let target = app.search_matches[app.search_cursor];
                        seek_to_offset(app, target);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn handle_mark(mode: MarkMode, letter: char, app: &mut ReaderApp) -> Result<()> {
    let Some(db) = app.db.as_ref() else {
        return Ok(());
    };
    let Some(book_id) = app.book_id else {
        return Ok(());
    };
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
                    seek_to_offset(app, target);
                }
                // Cross-spine jump is a no-op for v1; full support arrives with library integration.
            }
        }
    }
    Ok(())
}

fn save_highlight(app: &mut ReaderApp, start: usize, end: usize) -> anyhow::Result<()> {
    let Some(db) = app.db.as_ref() else {
        return Ok(());
    };
    let Some(book_id) = app.book_id else {
        return Ok(());
    };

    let chars: Vec<char> = app.plain_text.chars().collect();
    if end <= start || end > chars.len() {
        return Ok(());
    }

    let text: String = chars[start..end].iter().collect();
    let ctx_before_start = start.saturating_sub(80);
    let ctx_after_end = (end + 80).min(chars.len());
    let context_before: String = chars[ctx_before_start..start].iter().collect();
    let context_after: String = chars[end..ctx_after_end].iter().collect();

    let h = crate::store::highlights::Highlight {
        id: 0,
        book_id,
        spine_idx: app.spine_idx,
        chapter_title: None,
        char_offset_start: start as u64,
        char_offset_end: end as u64,
        text,
        context_before: Some(context_before),
        context_after: Some(context_after),
        note: None,
        anchor_status: crate::store::highlights::AnchorStatus::Ok,
    };

    let mut conn = db.conn()?;
    crate::store::highlights::insert(&mut conn, &h)?;
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
