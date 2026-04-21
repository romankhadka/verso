use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use rbook::Ebook;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{
    library::epub_meta,
    reader::{
        anchor,
        book::{self as readerbook, SpineData},
        page::Page,
        search::{self, SearchDirection},
    },
    store::{
        bookmarks::{self, Bookmark},
        db::Db,
        highlights::{self, AnchorStatus, Highlight},
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

/// Active modal overlay, if any.
#[derive(Debug, Clone)]
enum Modal {
    Toc {
        selected: usize,
    },
    Highlights {
        items: Vec<Highlight>,
        selected: usize,
    },
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
    pub spine_hrefs: Vec<String>,
    pub chapter_titles: Vec<String>,
    pub current_spine: u32,
    pub total_spines: u32,
    pub epub_path: PathBuf,
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
    cmd_mode: Option<String>,
    toast: Option<(String, Instant)>,
    modal: Option<Modal>,
    should_quit: bool,
}

const PROGRESS_PERSIST_INTERVAL: Duration = Duration::from_secs(5);
const TOAST_TTL: Duration = Duration::from_secs(3);

/// Simple wrapper for the `verso open <path>` CLI flow: opens an EPUB
/// without a database/book-id and with default keymap bindings.
pub fn run_with_epub(path: &Path) -> Result<()> {
    let title = epub_meta::extract(path)
        .map(|m| m.title)
        .unwrap_or_else(|_| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });
    run_with_epub_and_db(path, &title, None, None, None)
}

/// Primary reader entry point. Opens the EPUB at `path`, manages spine
/// navigation internally, and persists progress/bookmarks/highlights if a
/// database is supplied.
pub fn run_with_epub_and_db(
    path: &Path,
    title: &str,
    db: Option<Db>,
    book_id: Option<i64>,
    keymap_overrides: Option<&std::collections::BTreeMap<String, Vec<String>>>,
) -> Result<()> {
    let book = rbook::Epub::new(path)?;
    let spine_hrefs = readerbook::spine_hrefs(&book)?;
    let chapter_titles = readerbook::chapter_titles_from_book(&book);
    let total_spines = spine_hrefs.len() as u32;

    let entries = match keymap_overrides {
        Some(user) => defaults::merge_with_user(user),
        None => defaults::default_entries(),
    };
    let keymap = Keymap::from_config(&entries)?;

    let mut term = terminal::enter()?;
    let size = term.size()?;
    let col = 68u16.min(size.width);

    // Load spine 0 initially; `restore_progress` may reload a different spine.
    let initial = if total_spines == 0 {
        readerbook::load_spine_from_html("", col, size.height)
    } else {
        let html = book.read_file(&spine_hrefs[0])?;
        readerbook::load_spine_from_html(&html, col, size.height)
    };

    let mut app = ReaderApp {
        pages: initial.pages,
        page_idx: 0,
        row_idx: 0,
        column_width: col,
        theme: "dark".into(),
        chrome: Chrome::new(Duration::from_millis(3000)),
        title: title.to_string(),
        keymap,
        spine_hrefs,
        chapter_titles,
        current_spine: 0,
        total_spines,
        epub_path: path.to_path_buf(),
        book_id,
        db,
        mode: Mode::Normal,
        pending_mark: None,
        plain_text: initial.plain_text,
        plain_text_chars: initial.plain_text_chars,
        last_persist: Instant::now(),
        search_buffer: String::new(),
        search_mode: None,
        search_matches: Vec::new(),
        search_cursor: 0,
        cmd_mode: None,
        toast: None,
        modal: None,
        should_quit: false,
    };

    restore_progress(&mut app);

    let res = event_loop(&mut term, &mut app);
    terminal::leave(&mut term)?;
    res
}

/// Reopen the EPUB, load spine `idx`, repaginate, and reset page/search state.
/// The caller is responsible for deciding what to do with `page_idx` afterwards
/// (we reset to 0 here so chapter navigation lands on the first page).
fn load_spine(app: &mut ReaderApp, idx: u32) -> Result<()> {
    if idx >= app.total_spines {
        return Ok(());
    }
    let book = rbook::Epub::new(&app.epub_path)?;
    let data: SpineData =
        readerbook::load_spine_data(&book, idx as usize, app.column_width, terminal_height()?)?;
    app.pages = data.pages;
    app.plain_text = data.plain_text;
    app.plain_text_chars = data.plain_text_chars;
    app.page_idx = 0;
    app.current_spine = idx;
    app.search_matches.clear();
    app.search_cursor = 0;
    Ok(())
}

/// Query the terminal for its current height. Used during `load_spine`
/// to keep pagination in sync if the window has resized since open.
fn terminal_height() -> Result<u16> {
    let (_w, h) = crossterm::terminal::size()?;
    Ok(h)
}

/// Restore progress: jump to the stored spine first, then page-seek.
/// Clamps `spine_idx` into range if the EPUB was replaced with a shorter edition.
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
    drop(conn);

    let clamped = if app.total_spines == 0 {
        0
    } else {
        row.spine_idx.min(app.total_spines - 1)
    };
    if clamped != app.current_spine {
        if let Err(e) = load_spine(app, clamped) {
            tracing::warn!(error = %e, "restore_progress: load_spine failed");
            return;
        }
    }

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
        spine_idx: app.current_spine,
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
        spine_idx: app.current_spine,
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

fn set_toast(app: &mut ReaderApp, msg: impl Into<String>) {
    app.toast = Some((msg.into(), Instant::now()));
}

fn event_loop(term: &mut Tui, app: &mut ReaderApp) -> Result<()> {
    loop {
        if app.should_quit {
            break;
        }
        let now = Instant::now();
        if now.duration_since(app.last_persist) >= PROGRESS_PERSIST_INTERVAL {
            save_progress(app);
        }
        // Expire stale toasts before rendering.
        if let Some((_, t)) = &app.toast {
            if now.duration_since(*t) >= TOAST_TTL {
                app.toast = None;
            }
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
            render_status(f, chunks[1], app);

            if let Some(modal) = app.modal.clone() {
                render_modal(f, area, app, &modal);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                app.chrome.touch(Instant::now());

                // Modals swallow input before anything else.
                if app.modal.is_some() {
                    handle_modal_key(app, k)?;
                    continue;
                }

                // Handle pending mark follow-up letter (before keymap).
                if let Some(mode) = app.pending_mark {
                    if let KeyCode::Char(letter) = k.code {
                        if letter.is_ascii_alphabetic() {
                            handle_mark(mode, letter, app)?;
                            app.pending_mark = None;
                            continue;
                        }
                    }
                    app.pending_mark = None;
                    continue;
                }

                // Intercept search-prompt keys.
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

                // Intercept command-prompt keys.
                if app.cmd_mode.is_some() {
                    handle_cmd_key(app, k)?;
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
                    Dispatch::Fire(Action::NextChapter)
                        if app.current_spine + 1 < app.total_spines =>
                    {
                        if let Err(e) = load_spine(app, app.current_spine + 1) {
                            set_toast(app, format!("chapter load failed: {e}"));
                        }
                    }
                    Dispatch::Fire(Action::PrevChapter) if app.current_spine > 0 => {
                        if let Err(e) = load_spine(app, app.current_spine - 1) {
                            set_toast(app, format!("chapter load failed: {e}"));
                        }
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
                    Dispatch::Fire(Action::BeginCmd) => {
                        app.cmd_mode = Some(String::new());
                    }
                    Dispatch::Fire(Action::ListHighlights) => {
                        open_highlights_modal(app);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn render_status(f: &mut ratatui::Frame, area: Rect, app: &ReaderApp) {
    if app.search_mode.is_some() {
        let prefix = match app.search_mode {
            Some(SearchDirection::Backward) => "?",
            _ => "/",
        };
        let status = format!("{prefix}{}", app.search_buffer);
        f.render_widget(ratatui::widgets::Paragraph::new(status), area);
        return;
    }
    if let Some(buf) = &app.cmd_mode {
        let status = format!(":{buf}");
        f.render_widget(ratatui::widgets::Paragraph::new(status), area);
        return;
    }
    if let Some((msg, _)) = &app.toast {
        f.render_widget(ratatui::widgets::Paragraph::new(msg.clone()), area);
        return;
    }
    let mode_str = match app.mode {
        Mode::Visual { .. } => " [VIS] ",
        Mode::Normal => "",
    };
    let ch = chapter_label(app);
    let status = format!(
        "{} {} · {} · page {}/{} ",
        mode_str,
        app.title,
        ch,
        app.page_idx + 1,
        app.pages.len()
    );
    f.render_widget(ratatui::widgets::Paragraph::new(status), area);
}

fn chapter_label(app: &ReaderApp) -> String {
    let idx = app.current_spine as usize;
    app.chapter_titles
        .get(idx)
        .cloned()
        .unwrap_or_else(|| format!("Chapter {}", idx + 1))
}

fn handle_cmd_key(app: &mut ReaderApp, k: KeyEvent) -> Result<()> {
    // Safe to unwrap: caller guards `app.cmd_mode.is_some()`.
    match k.code {
        KeyCode::Char(c) => {
            if let Some(buf) = app.cmd_mode.as_mut() {
                buf.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Some(buf) = app.cmd_mode.as_mut() {
                buf.pop();
            }
        }
        KeyCode::Esc => {
            app.cmd_mode = None;
        }
        KeyCode::Enter => {
            let cmd = app.cmd_mode.take().unwrap_or_default();
            dispatch_command(app, cmd.trim())?;
        }
        _ => {}
    }
    Ok(())
}

fn dispatch_command(app: &mut ReaderApp, cmd: &str) -> Result<()> {
    match cmd {
        "" => {}
        "toc" => {
            app.modal = Some(Modal::Toc {
                selected: app.current_spine as usize,
            });
        }
        "hl" => {
            open_highlights_modal(app);
        }
        "export" => {
            run_export(app);
        }
        "w" => match force_save_progress(app) {
            Ok(()) => {}
            Err(e) => set_toast(app, format!("write failed: {e}")),
        },
        "q" => {
            save_progress(app);
            save_auto_bookmark(app)?;
            app.should_quit = true;
        }
        other => set_toast(app, format!("unknown command: {other}")),
    }
    Ok(())
}

fn force_save_progress(app: &mut ReaderApp) -> anyhow::Result<()> {
    let (Some(db), Some(book_id)) = (app.db.as_ref(), app.book_id) else {
        return Ok(());
    };
    let mut conn = db.conn()?;
    let char_offset = current_char_offset(app);
    let percent = if app.plain_text_chars == 0 {
        0.0
    } else {
        (char_offset as f32 / app.plain_text_chars as f32 * 100.0).clamp(0.0, 100.0)
    };
    let anchor_hash = anchor::anchor_hash(&app.plain_text, char_offset as usize);
    let row = ProgressRow {
        book_id,
        spine_idx: app.current_spine,
        char_offset,
        anchor_hash,
        percent,
        time_read_s: 0,
        words_read: 0,
    };
    progress::upsert(&mut conn, &row)?;
    app.last_persist = Instant::now();
    Ok(())
}

fn run_export(app: &mut ReaderApp) {
    let (Some(db), Some(book_id)) = (app.db.as_ref(), app.book_id) else {
        return;
    };
    match export_highlights(db, book_id, app) {
        Ok(path) => set_toast(app, format!("exported: {}", path.display())),
        Err(e) => set_toast(app, format!("export failed: {e}")),
    }
}

fn export_highlights(db: &Db, book_id: i64, app: &ReaderApp) -> anyhow::Result<PathBuf> {
    let conn = db.conn()?;
    let highs = highlights::list(&conn, book_id)?;
    let meta = epub_meta::extract(&app.epub_path)?;
    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Iso8601::DEFAULT)?;
    let ctx = crate::export::markdown::BookContext {
        title: meta.title.clone(),
        author: meta.author.clone(),
        published: meta.published_at.clone(),
        progress_pct: None,
        source_path: app.epub_path.display().to_string(),
        tags: vec![],
        exported_at: now,
    };
    let md = crate::export::markdown::render(&ctx, &highs);
    let export_dir = dirs_home_books_highlights();
    let slug = crate::export::writer::slug_from_title(&meta.title);
    crate::export::writer::write_export(&export_dir, &slug, &md)
}

fn dirs_home_books_highlights() -> PathBuf {
    let tilde = shellexpand::tilde("~/Books/highlights").to_string();
    PathBuf::from(tilde)
}

fn open_highlights_modal(app: &mut ReaderApp) {
    let (Some(db), Some(book_id)) = (app.db.as_ref(), app.book_id) else {
        set_toast(app, "no database — highlights unavailable");
        return;
    };
    let conn = match db.conn() {
        Ok(c) => c,
        Err(e) => {
            set_toast(app, format!("db open failed: {e}"));
            return;
        }
    };
    let items = match highlights::list(&conn, book_id) {
        Ok(v) => v,
        Err(e) => {
            set_toast(app, format!("list failed: {e}"));
            return;
        }
    };
    app.modal = Some(Modal::Highlights { items, selected: 0 });
}

fn handle_modal_key(app: &mut ReaderApp, k: KeyEvent) -> Result<()> {
    let modal = match app.modal.take() {
        Some(m) => m,
        None => return Ok(()),
    };
    match modal {
        Modal::Toc { mut selected } => match k.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // close
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if selected + 1 < app.chapter_titles.len() {
                    selected += 1;
                }
                app.modal = Some(Modal::Toc { selected });
            }
            KeyCode::Char('k') | KeyCode::Up => {
                selected = selected.saturating_sub(1);
                app.modal = Some(Modal::Toc { selected });
            }
            KeyCode::Enter => {
                if let Err(e) = load_spine(app, selected as u32) {
                    set_toast(app, format!("load failed: {e}"));
                }
            }
            _ => {
                app.modal = Some(Modal::Toc { selected });
            }
        },
        Modal::Highlights {
            mut items,
            mut selected,
        } => match k.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // close
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if selected + 1 < items.len() {
                    selected += 1;
                }
                app.modal = Some(Modal::Highlights { items, selected });
            }
            KeyCode::Char('k') | KeyCode::Up => {
                selected = selected.saturating_sub(1);
                app.modal = Some(Modal::Highlights { items, selected });
            }
            KeyCode::Enter => {
                if let Some(h) = items.get(selected).cloned() {
                    if let Err(e) = load_spine(app, h.spine_idx) {
                        set_toast(app, format!("load failed: {e}"));
                    } else {
                        seek_to_offset(app, h.char_offset_start as usize);
                    }
                }
            }
            KeyCode::Char('d') => {
                if let (Some(db), Some(h)) = (app.db.as_ref(), items.get(selected).cloned()) {
                    match db.conn() {
                        Ok(mut conn) => match highlights::delete(&mut conn, h.id) {
                            Ok(()) => {
                                items.remove(selected);
                                if selected >= items.len() && selected > 0 {
                                    selected -= 1;
                                }
                            }
                            Err(e) => set_toast(app, format!("delete failed: {e}")),
                        },
                        Err(e) => set_toast(app, format!("db open failed: {e}")),
                    }
                }
                app.modal = Some(Modal::Highlights { items, selected });
            }
            // Inline note editing is deferred to v1.1. Swallow the key
            // silently so it doesn't feel broken.
            KeyCode::Char('e') => {
                app.modal = Some(Modal::Highlights { items, selected });
            }
            _ => {
                app.modal = Some(Modal::Highlights { items, selected });
            }
        },
    }
    Ok(())
}

fn render_modal(f: &mut ratatui::Frame, area: Rect, app: &ReaderApp, modal: &Modal) {
    let panel = Rect {
        x: area.x + area.width / 6,
        y: area.y + area.height / 6,
        width: (area.width * 2 / 3).max(40),
        height: (area.height * 2 / 3).max(10),
    };
    f.render_widget(ratatui::widgets::Clear, panel);
    match modal {
        Modal::Toc { selected } => {
            let items: Vec<ratatui::widgets::ListItem> = app
                .chapter_titles
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let marker = if i == app.current_spine as usize {
                        "▸"
                    } else {
                        " "
                    };
                    let line = format!("{marker} {t}");
                    let mut item = ratatui::widgets::ListItem::new(line);
                    if i == *selected {
                        item = item.style(
                            ratatui::style::Style::default()
                                .add_modifier(ratatui::style::Modifier::REVERSED),
                        );
                    }
                    item
                })
                .collect();
            let block = ratatui::widgets::Block::default()
                .title(" Table of contents ")
                .borders(ratatui::widgets::Borders::ALL);
            let list = ratatui::widgets::List::new(items).block(block);
            f.render_widget(list, panel);
        }
        Modal::Highlights { items, selected } => {
            let rows: Vec<ratatui::widgets::ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let ch_label = app
                        .chapter_titles
                        .get(h.spine_idx as usize)
                        .cloned()
                        .unwrap_or_else(|| format!("Chapter {}", h.spine_idx + 1));
                    let preview = snippet(&h.text, 60);
                    let status = match h.anchor_status {
                        AnchorStatus::Ok => "ok",
                        AnchorStatus::Drifted => "drifted",
                        AnchorStatus::Lost => "lost",
                    };
                    let line = format!("{ch_label}  \"{preview}\"  [{status}]");
                    let mut item = ratatui::widgets::ListItem::new(line);
                    if i == *selected {
                        item = item.style(
                            ratatui::style::Style::default()
                                .add_modifier(ratatui::style::Modifier::REVERSED),
                        );
                    }
                    item
                })
                .collect();
            let title = format!(" Highlights ({}) ", items.len());
            let block = ratatui::widgets::Block::default()
                .title(title)
                .borders(ratatui::widgets::Borders::ALL);
            let list = ratatui::widgets::List::new(rows).block(block);
            f.render_widget(list, panel);
        }
    }
}

fn snippet(s: &str, max: usize) -> String {
    let trimmed: String = s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    if trimmed.chars().count() <= max {
        trimmed
    } else {
        let shortened: String = trimmed.chars().take(max.saturating_sub(1)).collect();
        format!("{shortened}…")
    }
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
                spine_idx: app.current_spine,
                char_offset: co,
                anchor_hash: anchor::anchor_hash(&app.plain_text, co as usize),
            };
            let mut conn = db.conn()?;
            bookmarks::set_bookmark(&mut conn, &bm)?;
        }
        MarkMode::Jump => {
            let conn = db.conn()?;
            if let Some(bm) = bookmarks::get_bookmark(&conn, book_id, &mark)? {
                drop(conn);
                if bm.spine_idx != app.current_spine {
                    if let Err(e) = load_spine(app, bm.spine_idx) {
                        set_toast(app, format!("mark load failed: {e}"));
                        return Ok(());
                    }
                }
                let target = bm.char_offset as usize;
                seek_to_offset(app, target);
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

    let chapter_title = app.chapter_titles.get(app.current_spine as usize).cloned();

    let h = Highlight {
        id: 0,
        book_id,
        spine_idx: app.current_spine,
        chapter_title,
        char_offset_start: start as u64,
        char_offset_end: end as u64,
        text,
        context_before: Some(context_before),
        context_after: Some(context_after),
        note: None,
        anchor_status: AnchorStatus::Ok,
    };

    let mut conn = db.conn()?;
    highlights::insert(&mut conn, &h)?;
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
