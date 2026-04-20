use anyhow::Result;
use rbook::Ebook;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_at: Option<String>,
    pub stable_id: Option<String>,
    pub word_count: Option<u64>,
    pub spine_items: usize,
}

pub fn extract(path: &Path) -> Result<Meta> {
    let book = rbook::Epub::new(path)?;
    let m = book.metadata();

    let title = m.title().map(|s| s.value().to_string()).unwrap_or_default();
    let author = m.creators().first().map(|c| c.value().to_string());
    let language = m.language().map(|s| s.value().to_string());
    let publisher = m.publisher().first().map(|s| s.value().to_string());
    let published_at = m.date().map(|s| s.value().to_string());
    let stable_id = m.unique_identifier().map(|s| s.value().to_string());

    let spine_elements = book.spine().elements();
    let spine_items = spine_elements.len();

    let mut words: u64 = 0;
    for el in &spine_elements {
        if let Some(item) = book.manifest().by_id(el.name()) {
            if let Ok(content) = book.read_file(item.value()) {
                words += count_words(&content);
            }
        }
    }

    Ok(Meta {
        title,
        author,
        language,
        publisher,
        published_at,
        stable_id,
        word_count: Some(words),
        spine_items,
    })
}

fn count_words(html: &str) -> u64 {
    // Cheap estimate: strip tags, whitespace-split.
    let text = strip_tags(html);
    text.split_whitespace().count() as u64
}

fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}
