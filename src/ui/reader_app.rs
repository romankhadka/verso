use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use std::time::{Duration, Instant};

use crate::{
    reader::{page::Page, page, sanitize, styled},
    ui::{
        chrome::{Chrome, ChromeState},
        keymap::{Action, defaults, table::{Dispatch, Keymap}},
        reader_view::ReaderView,
        terminal::{self, Tui},
    },
};

pub struct ReaderApp {
    pub pages: Vec<Page>,
    pub page_idx: usize,
    pub row_idx: usize,
    pub column_width: u16,
    pub theme: String,
    pub chrome: Chrome,
    pub title: String,
    pub keymap: Keymap,
}

pub fn run_with_html(html: &str, title: &str) -> Result<()> {
    let safe = sanitize::clean(html);
    let spans = styled::to_spans(&safe);

    let mut term = terminal::enter()?;
    let size = term.size()?;
    let col = 68u16.min(size.width);
    let pages = page::paginate(&spans, col, size.height.saturating_sub(2));
    let keymap = Keymap::from_config(&defaults::default_entries())?;

    let mut app = ReaderApp {
        pages, page_idx: 0, row_idx: 0, column_width: col,
        theme: "dark".into(), chrome: Chrome::new(Duration::from_millis(3000)),
        title: title.to_string(), keymap,
    };

    let res = event_loop(&mut term, &mut app);
    terminal::leave(&mut term)?;
    res
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
                    _ => {}
                }
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
