use verso::{library::scan, store::db::Db};

#[test]
fn scans_folder_and_inserts_books() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::copy(
        "tests/fixtures/time-machine.epub",
        tmp.path().join("tm.epub"),
    )
    .unwrap();

    let dbfile = tmp.path().join("verso.db");
    let db = Db::open(&dbfile).unwrap();
    db.migrate().unwrap();

    let report = scan::scan_folder(tmp.path(), &db).unwrap();
    assert_eq!(report.inserted, 1);
    assert_eq!(report.errors.len(), 0);

    let c = db.conn().unwrap();
    let n: i64 = c
        .query_row("SELECT COUNT(*) FROM books", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
}
