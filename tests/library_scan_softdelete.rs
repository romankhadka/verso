use verso::{library::scan, store::db::Db};

#[test]
fn soft_deletes_vanished_books() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::copy("tests/fixtures/time-machine.epub", tmp.path().join("tm.epub")).unwrap();

    let dbfile = tmp.path().join("verso.db");
    let db = Db::open(&dbfile).unwrap();
    db.migrate().unwrap();

    // First scan — book imported.
    scan::scan_folder(tmp.path(), &db).unwrap();
    let c = db.conn().unwrap();
    let n_before: i64 = c.query_row(
        "SELECT COUNT(*) FROM books WHERE deleted_at IS NULL", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(n_before, 1);

    // Remove the file and re-scan.
    std::fs::remove_file(tmp.path().join("tm.epub")).unwrap();
    scan::scan_folder(tmp.path(), &db).unwrap();

    let c = db.conn().unwrap();
    let n_after_active: i64 = c.query_row(
        "SELECT COUNT(*) FROM books WHERE deleted_at IS NULL", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(n_after_active, 0);
    let n_after_total: i64 = c.query_row(
        "SELECT COUNT(*) FROM books", [], |r| r.get(0)
    ).unwrap();
    assert_eq!(n_after_total, 1, "book should still exist with deleted_at set");
}
