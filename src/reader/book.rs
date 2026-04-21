//! Pure helpers for navigating an open EPUB: spine-item loading and
//! table-of-contents resolution. Kept free of any TUI dependency so
//! they can be unit-tested without a terminal.

use anyhow::{anyhow, Result};
use rbook::Epub;

use super::{page, page::Page, sanitize, styled};

/// Raw data describing a single spine item.
pub struct SpineData {
    /// Paginated rows at the given width/height.
    pub pages: Vec<Page>,
    /// Flat plain text of the sanitised chapter HTML.
    pub plain_text: String,
    /// Char count of `plain_text`.
    pub plain_text_chars: usize,
}

/// Collect manifest hrefs for every spine item, in order.
pub fn spine_hrefs(book: &Epub) -> Result<Vec<String>> {
    let spine = book.spine().elements();
    let mut out: Vec<String> = Vec::with_capacity(spine.len());
    for el in &spine {
        let idref = el.name();
        let item = book
            .manifest()
            .by_id(idref)
            .ok_or_else(|| anyhow!("manifest missing idref {}", idref))?;
        out.push(item.value().to_string());
    }
    Ok(out)
}

/// Compute human-readable titles per spine item.
///
/// Looks up each spine href in the TOC (NAV/NCX). Fragments are stripped
/// when matching (`ch04.xhtml#heading` matches `ch04.xhtml`). Spine items
/// without a TOC entry fall back to `"Chapter {i+1}"`.
pub fn chapter_titles_from_book(book: &Epub) -> Vec<String> {
    let spine_hrefs_result = spine_hrefs(book);
    let hrefs = match spine_hrefs_result {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    // Build TOC href -> label, matching on the path portion only.
    let toc_entries: Vec<(String, String)> = book
        .toc()
        .elements_flat()
        .into_iter()
        .filter_map(|e| {
            let label = e.name().trim().to_string();
            let href = e.value().trim().to_string();
            if label.is_empty() || href.is_empty() {
                None
            } else {
                Some((strip_fragment(&href).to_string(), label))
            }
        })
        .collect();

    hrefs
        .iter()
        .enumerate()
        .map(|(i, href)| {
            let href_path = strip_fragment(href);
            // Prefer full-path match; fall back to basename match for TOC
            // entries that use relative hrefs without directory prefixes.
            let exact = toc_entries
                .iter()
                .find(|(h, _)| h == href_path)
                .map(|(_, l)| l.clone());
            let by_base = exact.or_else(|| {
                let target_base = basename(href_path);
                toc_entries
                    .iter()
                    .find(|(h, _)| basename(h) == target_base)
                    .map(|(_, l)| l.clone())
            });
            by_base.unwrap_or_else(|| format!("Chapter {}", i + 1))
        })
        .collect()
}

/// Read the spine item at `idx`, sanitise and paginate to `width` x `height`.
pub fn load_spine_data(book: &Epub, idx: usize, width: u16, height: u16) -> Result<SpineData> {
    let hrefs = spine_hrefs(book)?;
    let href = hrefs
        .get(idx)
        .ok_or_else(|| anyhow!("spine index {} out of bounds (len {})", idx, hrefs.len()))?;
    let html = book.read_file(href)?;
    Ok(load_spine_from_html(&html, width, height))
}

/// Same as `load_spine_data` but takes pre-fetched HTML. Used by the
/// html-only entry point in the CLI.
pub fn load_spine_from_html(html: &str, width: u16, height: u16) -> SpineData {
    let safe = sanitize::clean(html);
    let spans = styled::to_spans(&safe);
    let plain_text: String = spans
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .concat();
    let pages = page::paginate(&spans, width, height.saturating_sub(2));
    let plain_text_chars = plain_text.chars().count();
    SpineData {
        pages,
        plain_text,
        plain_text_chars,
    }
}

fn strip_fragment(s: &str) -> &str {
    match s.find('#') {
        Some(i) => &s[..i],
        None => s,
    }
}

fn basename(s: &str) -> &str {
    match s.rfind('/') {
        Some(i) => &s[i + 1..],
        None => s,
    }
}
