use verso::reader::page::{paginate, PageRow};

#[test]
fn paginates_within_page_height() {
    let spans = verso::reader::styled::to_spans("<p>Lorem ipsum dolor sit amet.</p>".repeat(80).as_str());
    let pages = paginate(&spans, 50, 20);
    for (i, p) in pages.iter().enumerate() {
        assert!(p.rows.len() <= 20, "page {i} exceeds height: {}", p.rows.len());
    }
    assert!(pages.len() >= 3);
}

#[test]
fn empty_input_yields_one_empty_page() {
    let pages = paginate(&[], 50, 20);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].rows.len(), 0);
}
