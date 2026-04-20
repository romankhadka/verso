use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};
use crate::store::library_view::Row as LibRow;

pub struct LibraryView<'a> {
    pub rows: &'a [LibRow],
    pub selected: usize,
    pub sort_label: &'a str,
    pub filter_label: &'a str,
}

impl<'a> LibraryView<'a> {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let header = Row::new(vec!["Title", "Author", "Pages", "Progress", "Left", "Last"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let body: Vec<Row> = self.rows.iter().map(|r| {
            let pct = r.progress_pct.unwrap_or(0.0);
            let bar = render_bar(pct, 6);
            Row::new(vec![
                r.title.clone(),
                r.author.clone().unwrap_or_default(),
                r.pages.map(|p| p.to_string()).unwrap_or_default(),
                format!("{bar} {pct:>3.0}%"),
                format_time_left(r.time_left_s),
                r.last_read_at.clone().unwrap_or_else(|| "—".into()),
            ])
        }).collect();

        let widths = [
            Constraint::Min(20), Constraint::Length(16), Constraint::Length(6),
            Constraint::Length(13), Constraint::Length(6), Constraint::Length(10),
        ];
        let title = format!(" verso · Library · {} books · {} ",
                            self.rows.len(),
                            reading_count(self.rows));
        let block = Block::default().title(title).borders(Borders::ALL);
        let mut state = TableState::default(); state.select(Some(self.selected));
        let table = Table::new(body, widths).header(header).block(block)
            .highlight_style(Style::default().bg(Color::DarkGray));
        f.render_stateful_widget(table, area, &mut state);
    }
}

fn reading_count(rows: &[LibRow]) -> usize {
    rows.iter().filter(|r| r.finished_at.is_none() && r.progress_pct.unwrap_or(0.0) > 0.0).count()
}

fn render_bar(pct: f32, width: u16) -> String {
    let filled = (pct / 100.0 * width as f32).round() as usize;
    let empty  = (width as usize).saturating_sub(filled);
    "█".repeat(filled) + &"░".repeat(empty)
}

fn format_time_left(s: Option<u64>) -> String {
    match s {
        None => "—".into(),
        Some(secs) if secs < 3600 => format!("{}m", secs / 60),
        Some(secs) => format!("{}h", secs / 3600),
    }
}
