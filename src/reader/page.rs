use super::{linebreak, styled::Span};

#[derive(Debug, Clone)]
pub struct PageRow {
    pub text: String,
    pub spans: Vec<Span>,      // spans that intersect this row (for styling)
    pub char_offset: usize,    // offset of the first char on this row
}

#[derive(Debug, Clone)]
pub struct Page {
    pub rows: Vec<PageRow>,
}

/// Paginate a list of spans to pages of `height` rows at `width` columns.
/// In v1 styled spans are rendered as plain text for line-breaking;
/// full per-span styling on the output rows arrives in Task 24.
pub fn paginate(spans: &[Span], width: u16, height: u16) -> Vec<Page> {
    let height = height as usize;
    if spans.is_empty() { return vec![Page { rows: vec![] }]; }

    // 1) Flatten spans into plain text with a parallel offset map.
    let mut text = String::new();
    for s in spans { text.push_str(&s.text); }
    let lines = linebreak::wrap(&text, width);

    // 2) Map each line to its starting char_offset (best-effort: find() from running cursor).
    let mut rows: Vec<PageRow> = Vec::with_capacity(lines.len());
    let mut cursor = 0usize;
    for l in &lines {
        if l.is_empty() {
            rows.push(PageRow { text: String::new(), spans: vec![], char_offset: cursor });
            continue;
        }
        let off = text[cursor..].find(l.as_str()).map(|b| cursor + b).unwrap_or(cursor);
        let char_off = text[..off].chars().count();
        rows.push(PageRow { text: l.clone(), spans: vec![], char_offset: char_off });
        cursor = off + l.len();
    }

    // 3) Chunk into pages.
    rows.chunks(height).map(|c| Page { rows: c.to_vec() }).collect()
}
