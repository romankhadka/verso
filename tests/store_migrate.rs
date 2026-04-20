use verso::store::db::Db;

#[test]
fn migrations_apply_and_tables_exist() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();

    let tables: Vec<String> = db
        .conn()
        .unwrap()
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    for t in [
        "book_tags",
        "bookmarks",
        "books",
        "highlights",
        "progress",
        "tags",
    ] {
        assert!(
            tables.iter().any(|n| n == t),
            "missing table {t}: {tables:?}"
        );
    }
}

#[test]
fn pragmas_set_correctly() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    let c = db.conn().unwrap();
    let jm: String = c
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(jm.to_lowercase(), "wal");
    let fk: i64 = c
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fk, 1);
}
