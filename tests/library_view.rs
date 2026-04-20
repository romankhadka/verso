use verso::store::{db::Db, books::{BookRow, upsert}, library_view::{list_rows, Sort, Filter}};

#[test]
fn lists_rows_with_defaults() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    upsert(&mut db.conn().unwrap(), &BookRow::new_fixture("a")).unwrap();
    upsert(&mut db.conn().unwrap(), &BookRow::new_fixture("b")).unwrap();
    let rows = list_rows(&db.conn().unwrap(), Sort::LastRead, Filter::All).unwrap();
    assert_eq!(rows.len(), 2);
    std::mem::forget(tmp);
}
