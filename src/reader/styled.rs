use scraper::{Html, Node};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: bool,
    pub heading: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
    /// character offset into the plain-text extraction of this spine item.
    pub char_offset: usize,
}

pub fn to_spans(html: &str) -> Vec<Span> {
    let doc = Html::parse_document(html);
    let mut offset = 0usize;
    let mut out = Vec::new();
    walk(doc.root_element(), Style::default(), &mut offset, &mut out);
    out
}

fn walk(node: scraper::ElementRef, style: Style, offset: &mut usize, out: &mut Vec<Span>) {
    for child in node.children() {
        match child.value() {
            Node::Text(t) => {
                let text = t.to_string();
                if text.is_empty() { continue; }
                let len = text.chars().count();
                out.push(Span { text, style: style.clone(), char_offset: *offset });
                *offset += len;
            }
            Node::Element(el) => {
                let name = el.name();
                if matches!(name, "script" | "style" | "iframe" | "object" | "embed") { continue; }
                let mut s = style.clone();
                match name {
                    "em" | "i" => s.italic = true,
                    "strong" | "b" => s.bold = true,
                    "code" | "kbd" | "samp" => s.code = true,
                    "a" => s.link = true,
                    "h1" => s.heading = Some(1),
                    "h2" => s.heading = Some(2),
                    "h3" => s.heading = Some(3),
                    "h4" => s.heading = Some(4),
                    "h5" => s.heading = Some(5),
                    "h6" => s.heading = Some(6),
                    _ => {}
                }
                if let Some(er) = scraper::ElementRef::wrap(child) { walk(er, s, offset, out); }
            }
            _ => {}
        }
    }
}
