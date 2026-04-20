use verso::store::{
    bookmarks::{get_bookmark, set_bookmark, Bookmark},
    books::{upsert, BookRow},
    db::Db,
};

fn fresh() -> (Db, i64) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    let id = upsert(&mut db.conn().unwrap(), &BookRow::new_fixture("tm")).unwrap();
    std::mem::forget(tmp);
    (db, id)
}

#[test]
fn sets_and_reads_bookmark() {
    let (db, bid) = fresh();
    let b = Bookmark {
        book_id: bid,
        mark: "a".into(),
        spine_idx: 2,
        char_offset: 500,
        anchor_hash: "xx".into(),
    };
    set_bookmark(&mut db.conn().unwrap(), &b).unwrap();
    let got = get_bookmark(&db.conn().unwrap(), bid, "a")
        .unwrap()
        .unwrap();
    assert_eq!(got.spine_idx, 2);
    assert_eq!(got.char_offset, 500);
}
