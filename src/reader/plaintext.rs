use scraper::{Html, Selector};

/// Produce the canonical plain-text extraction used for the location model.
/// Must be deterministic and stable — `char_offset` semantics depend on it.
pub fn from_html(html: &str) -> String {
    // Parse once; traverse in document order.
    let doc = Html::parse_document(html);
    let body_sel = Selector::parse("body, html").unwrap();

    let mut out = String::new();
    for root in doc.select(&body_sel) {
        walk(root, &mut out);
        break; // first body element only
    }
    if out.is_empty() {
        // Fallback: some EPUB chapters are fragments without a body.
        walk(doc.root_element(), &mut out);
    }
    normalise_whitespace(&out)
}

fn walk(node: scraper::ElementRef, out: &mut String) {
    use scraper::Node;
    for child in node.children() {
        match child.value() {
            Node::Text(t) => out.push_str(&collapse_spaces(t)),
            Node::Element(el) => {
                let name = el.name();
                if matches!(name, "script" | "style" | "iframe" | "object" | "embed") {
                    continue;
                }
                let is_block = matches!(
                    name,
                    "p" | "div" | "br" | "h1"|"h2"|"h3"|"h4"|"h5"|"h6"
                      | "li" | "blockquote" | "pre" | "tr" | "hr" | "figure" | "figcaption"
                );
                if is_block && !out.ends_with('\n') { out.push('\n'); }
                if let Some(er) = scraper::ElementRef::wrap(child) { walk(er, out); }
                if matches!(name, "p" | "h1"|"h2"|"h3"|"h4"|"h5"|"h6" | "blockquote" | "pre" | "figure" ) {
                    if !out.ends_with("\n\n") { out.push('\n'); }
                }
            }
            _ => {}
        }
    }
}

fn normalise_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_blank = false;
    for line in s.split('\n') {
        let trimmed = collapse_spaces(line.trim_end());
        if trimmed.is_empty() {
            if !last_blank && !out.is_empty() {
                out.push_str("\n\n");
                last_blank = true;
            }
        } else {
            if !out.is_empty() && !out.ends_with("\n\n") && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&trimmed);
            last_blank = false;
        }
    }
    out.trim().to_string()
}

fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space { out.push(' '); }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}
