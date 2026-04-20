use crate::store::highlights::{AnchorStatus, Highlight};

pub struct BookContext {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub progress_pct: Option<f32>,
    pub source_path: String,
    pub tags: Vec<String>,
    pub exported_at: String,
}

pub fn render(ctx: &BookContext, highs: &[Highlight]) -> String {
    let mut s = String::new();
    s.push_str("---\n");
    s.push_str(&format!("title: {}\n", ctx.title));
    if let Some(a) = &ctx.author {
        s.push_str(&format!("author: {a}\n"));
    }
    if let Some(p) = &ctx.published {
        s.push_str(&format!("published: {p}\n"));
    }
    s.push_str(&format!("exported: {}\n", ctx.exported_at));
    if let Some(p) = ctx.progress_pct {
        s.push_str(&format!("progress: {p:.0}%\n"));
    }
    s.push_str(&format!("source: {}\n", ctx.source_path));
    if !ctx.tags.is_empty() {
        s.push_str(&format!("tags: [{}]\n", ctx.tags.join(", ")));
    }
    s.push_str("---\n\n");

    let mut current_chapter: Option<String> = None;
    for h in highs {
        if h.chapter_title.as_deref() != current_chapter.as_deref() {
            current_chapter = h.chapter_title.clone();
            if let Some(ch) = &current_chapter {
                s.push_str(&format!("## {ch}\n\n"));
            }
        }
        let marker = if matches!(h.anchor_status, AnchorStatus::Drifted) {
            " *(drifted)*"
        } else if matches!(h.anchor_status, AnchorStatus::Lost) {
            " *(lost)*"
        } else {
            ""
        };
        s.push_str(&format!("> {}{}\n\n", h.text.replace('\n', " "), marker));
        if let Some(n) = &h.note {
            s.push_str(&format!("**Note:** {n}\n\n"));
        }
        s.push_str("---\n\n");
    }
    s
}
