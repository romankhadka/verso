use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use std::time::{Duration, Instant};

use crate::{
    reader::{page::Page, page, sanitize, styled},
    ui::{chrome::{Chrome, ChromeState}, reader_view::ReaderView, terminal::{self, Tui}},
};

pub struct ReaderApp {
    pub pages: Vec<Page>,
    pub page_idx: usize,
    pub row_idx: usize,
    pub column_width: u16,
    pub theme: String,
    pub chrome: Chrome,
    pub title: String,
}

pub fn run_with_html(html: &str, title: &str) -> Result<()> {
    let safe = sanitize::clean(html);
    let spans = styled::to_spans(&safe);

    let mut term = terminal::enter()?;
    let size = term.size()?;
    let col = 68u16.min(size.width);
    let pages = page::paginate(&spans, col, size.height.saturating_sub(2));

    let mut app = ReaderApp {
        pages, page_idx: 0, row_idx: 0, column_width: col,
        theme: "dark".into(), chrome: Chrome::new(Duration::from_millis(3000)),
        title: title.to_string(),
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
            // minimal chrome: always show a thin status line
            let status = format!(" {} · page {}/{} ", app.title, app.page_idx + 1, app.pages.len());
            f.render_widget(ratatui::widgets::Paragraph::new(status), chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                app.chrome.touch(Instant::now());
                match k.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.page_idx + 1 < app.pages.len() { app.page_idx += 1; }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.page_idx > 0 { app.page_idx -= 1; }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
