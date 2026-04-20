use verso::library::epub_meta;

#[test]
fn parses_time_machine() {
    let meta = epub_meta::extract(std::path::Path::new("tests/fixtures/time-machine.epub")).unwrap();
    assert!(meta.title.contains("Time Machine"));
    assert!(meta.author.as_deref().unwrap_or("").to_lowercase().contains("wells"));
    assert!(meta.stable_id.is_some(), "EPUB identifier missing");
    assert!(meta.word_count.unwrap_or(0) > 5000, "word count suspiciously low: {:?}", meta.word_count);
    assert!(meta.spine_items >= 3);
}
