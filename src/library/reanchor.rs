use crate::{
    reader::{anchor, plaintext},
    store::{
        db::Db,
        highlights::{self, AnchorStatus},
    },
};
use rbook::Ebook;
use std::path::Path;

/// For every highlight of the given book, re-compute its location against the current EPUB.
/// Updates `anchor_status` and offsets in place.
pub fn reanchor_book(db: &Db, book_id: i64, epub_path: &Path) -> anyhow::Result<()> {
    let conn = db.conn()?;
    let highlights = highlights::list(&conn, book_id)?;
    if highlights.is_empty() {
        return Ok(());
    }

    let book = rbook::Epub::new(epub_path)?;
    let spine = book.spine().elements();
    let spine_items: Vec<(String, String)> = spine
        .iter()
        .filter_map(|el| {
            let idref = el.name().to_string();
            let href = book
                .manifest()
                .by_id(&idref)
                .map(|m| m.value().to_string())?;
            Some((idref, href))
        })
        .collect();

    let mut conn = db.conn()?;
    let tx = conn.transaction()?;
    for h in highlights {
        let Some((_, href)) = spine_items.get(h.spine_idx as usize) else {
            continue;
        };
        let html = match book.read_file(href) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let text = plaintext::from_html(&html);
        let ctx_b = h.context_before.as_deref().unwrap_or("");
        let ctx_a = h.context_after.as_deref().unwrap_or("");
        let maybe_hit =
            anchor::reanchor(&text, &h.text, h.char_offset_start as usize, ctx_b, ctx_a);
        let (start, end, status) = match maybe_hit {
            Some(off) => (
                off as u64,
                off as u64 + h.text.chars().count() as u64,
                AnchorStatus::Ok,
            ),
            None if text.contains(&h.text) => {
                let fallback = text.find(&h.text).unwrap();
                let char_off = text[..fallback].chars().count() as u64;
                (
                    char_off,
                    char_off + h.text.chars().count() as u64,
                    AnchorStatus::Drifted,
                )
            }
            None => (h.char_offset_start, h.char_offset_end, AnchorStatus::Lost),
        };
        tx.execute(
            "UPDATE highlights SET char_offset_start=?, char_offset_end=?, anchor_status=?, updated_at=CURRENT_TIMESTAMP WHERE id=?",
            rusqlite::params![start, end, status.as_str(), h.id],
        )?;
    }
    tx.commit()?;
    Ok(())
}
