use verso::{
    export::markdown::render,
    store::highlights::{AnchorStatus, Highlight},
};

#[test]
fn renders_frontmatter_and_quotes() {
    let highs = vec![Highlight {
        id: 1,
        book_id: 1,
        spine_idx: 3,
        chapter_title: Some("Chapter 4".into()),
        char_offset_start: 100,
        char_offset_end: 150,
        text: "A beginning is the time...".into(),
        context_before: None,
        context_after: None,
        note: Some("Irulan's epigraph".into()),
        anchor_status: AnchorStatus::Ok,
    }];
    let out = render(
        &verso::export::markdown::BookContext {
            title: "Dune".into(),
            author: Some("Frank Herbert".into()),
            published: Some("1965".into()),
            progress_pct: Some(12.0),
            source_path: "/tmp/dune.epub".into(),
            tags: vec!["sci-fi".into()],
            exported_at: "2026-04-20T14:32:00Z".into(),
        },
        &highs,
    );
    insta::assert_snapshot!(out);
}
