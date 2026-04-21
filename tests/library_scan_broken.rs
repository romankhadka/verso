use verso::{library::scan, store::db::Db};

#[test]
fn malformed_epub_appears_in_broken_filter() {
    use verso::store::library_view::{list_rows, Filter, Sort};
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("bogus.epub"), b"not a real epub").unwrap();

    let dbfile = tmp.path().join("verso.db");
    let db = Db::open(&dbfile).unwrap();
    db.migrate().unwrap();
    let report = scan::scan_folder(tmp.path(), &db).unwrap();
    assert_eq!(report.errors.len(), 1, "should have reported 1 parse error");

    let c = db.conn().unwrap();
    let broken = list_rows(&c, Sort::LastRead, Filter::Broken).unwrap();
    assert_eq!(broken.len(), 1);
    assert_eq!(broken[0].title, "bogus.epub");
    assert!(broken[0].parse_error.is_some());

    let all = list_rows(&c, Sort::LastRead, Filter::All).unwrap();
    assert_eq!(all.len(), 1, "broken book should also appear in 'all'");
}
