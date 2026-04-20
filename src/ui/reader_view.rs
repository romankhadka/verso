use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span as TuiSpan},
    widgets::{Paragraph, Wrap},
    Frame,
};
use crate::reader::page::Page;

pub struct ReaderView<'a> {
    pub page: Option<&'a Page>,
    pub column_width: u16,
    pub theme: &'a str,
}

impl<'a> ReaderView<'a> {
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let bg = if self.theme == "light" { Color::White } else { Color::Reset };
        let fg = if self.theme == "light" { Color::Black } else { Color::Gray };

        let left_pad = area.width.saturating_sub(self.column_width) / 2;
        let text_area = Rect {
            x: area.x + left_pad,
            y: area.y,
            width: self.column_width.min(area.width),
            height: area.height,
        };

        let lines: Vec<Line> = match self.page {
            Some(p) => p.rows.iter().map(|r| Line::from(vec![TuiSpan::styled(r.text.clone(), Style::default().fg(fg).bg(bg))])).collect(),
            None => vec![Line::from("…paginating")],
        };
        let para = Paragraph::new(lines).alignment(Alignment::Left).wrap(Wrap { trim: false });
        f.render_widget(para, text_area);
    }
}
