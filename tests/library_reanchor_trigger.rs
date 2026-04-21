use verso::{
    library::scan,
    store::{
        db::Db,
        highlights::{self, AnchorStatus, Highlight},
    },
};

/// Copy `src` to `dst`, adding an extra file inside the archive. This yields a
/// structurally-valid EPUB with all original entries plus a marker entry, so
/// the file_hash changes but the book still parses and contains all prior text.
fn rebuild_epub_with_marker(src: &std::path::Path, dst: &std::path::Path) {
    use std::io::{Read, Write};
    let bytes = std::fs::read(src).unwrap();
    let cursor = std::io::Cursor::new(bytes);
    let mut src_zip = zip::ZipArchive::new(cursor).unwrap();

    let dst_file = std::fs::File::create(dst).unwrap();
    let mut dst_zip = zip::ZipWriter::new(dst_file);

    for i in 0..src_zip.len() {
        let mut entry = src_zip.by_index(i).unwrap();
        let name = entry.name().to_string();
        let opts = zip::write::FileOptions::default().compression_method(entry.compression());
        dst_zip.start_file(name, opts).unwrap();
        let mut content = Vec::new();
        entry.read_to_end(&mut content).unwrap();
        dst_zip.write_all(&content).unwrap();
    }

    dst_zip
        .start_file("VERSO_TEST_MARKER.txt", zip::write::FileOptions::default())
        .unwrap();
    dst_zip.write_all(b"rehash").unwrap();
    dst_zip.finish().unwrap();
}

#[test]
fn reanchor_fires_when_hash_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let book_path = tmp.path().join("tm.epub");
    std::fs::copy("tests/fixtures/time-machine.epub", &book_path).unwrap();

    let dbfile = tmp.path().join("verso.db");
    let db = Db::open(&dbfile).unwrap();
    db.migrate().unwrap();

    // First scan imports.
    scan::scan_folder(tmp.path(), &db).unwrap();
    let bid: i64 = db
        .conn()
        .unwrap()
        .query_row("SELECT id FROM books", [], |r| r.get(0))
        .unwrap();

    // Plant a highlight with an absurd offset to verify it's corrected after
    // reanchor. "The Time Traveller" appears repeatedly in H. G. Wells's text.
    let needle = "The Time Traveller";
    let h = Highlight {
        id: 0,
        book_id: bid,
        spine_idx: 2,
        chapter_title: None,
        char_offset_start: 9_999_999,
        char_offset_end: 9_999_999 + needle.chars().count() as u64,
        text: needle.into(),
        context_before: Some(String::new()),
        context_after: Some(String::new()),
        note: None,
        anchor_status: AnchorStatus::Lost,
    };
    highlights::insert(&mut db.conn().unwrap(), &h).unwrap();

    // Overwrite the EPUB with a rebuilt copy whose bytes (and therefore hash)
    // differ but whose text content remains readable.
    let rebuilt = tmp.path().join("tm-rebuilt.epub");
    rebuild_epub_with_marker(&book_path, &rebuilt);
    std::fs::rename(&rebuilt, &book_path).unwrap();

    scan::scan_folder(tmp.path(), &db).unwrap();

    // Verify reanchor fired: status should promote from Lost to Ok/Drifted,
    // and the absurd offset should be replaced with a reasonable one.
    let rows = highlights::list(&db.conn().unwrap(), bid).unwrap();
    assert_eq!(rows.len(), 1);
    assert!(
        matches!(
            rows[0].anchor_status,
            AnchorStatus::Ok | AnchorStatus::Drifted
        ),
        "expected Ok or Drifted after reanchor, got {:?}",
        rows[0].anchor_status
    );
    assert!(
        rows[0].char_offset_start < 1_000_000,
        "expected offset to be corrected below 1_000_000, got {}",
        rows[0].char_offset_start
    );
}
