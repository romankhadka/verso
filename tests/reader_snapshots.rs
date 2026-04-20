#[test]
fn time_machine_chapter_1_at_68_dark() {
    use rbook::Ebook;
    let book =
        rbook::Epub::new(std::path::Path::new("tests/fixtures/time-machine.epub")).unwrap();
    let spine = book.spine().elements();
    let first = spine.first().unwrap();
    let idref = first.name();
    let manifest_item = book.manifest().by_id(idref).unwrap();
    let html = book.read_file(manifest_item.value()).unwrap();
    let safe = verso::reader::sanitize::clean(&html);
    let spans = verso::reader::styled::to_spans(&safe);
    let pages = verso::reader::page::paginate(&spans, 68, 40);
    let rendered: String = pages
        .iter()
        .take(1)
        .flat_map(|p| p.rows.iter().map(|r| r.text.clone() + "\n"))
        .collect();
    insta::assert_snapshot!(rendered);
}
