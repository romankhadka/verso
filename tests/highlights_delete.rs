use verso::store::{
    books::{upsert, BookRow},
    db::Db,
    highlights::{delete, insert, list, AnchorStatus, Highlight},
};

fn fresh() -> (Db, i64) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    let id = upsert(&mut db.conn().unwrap(), &BookRow::new_fixture("tm")).unwrap();
    std::mem::forget(tmp);
    (db, id)
}

fn mkhl(book_id: i64) -> Highlight {
    Highlight {
        id: 0,
        book_id,
        spine_idx: 1,
        chapter_title: Some("Ch.1".into()),
        char_offset_start: 100,
        char_offset_end: 110,
        text: "Hello hi".into(),
        context_before: Some("pre".into()),
        context_after: Some("post".into()),
        note: None,
        anchor_status: AnchorStatus::Ok,
    }
}

#[test]
fn insert_then_delete_round_trip() {
    let (db, bid) = fresh();
    let new_id = insert(&mut db.conn().unwrap(), &mkhl(bid)).unwrap();
    assert_eq!(list(&db.conn().unwrap(), bid).unwrap().len(), 1);
    delete(&mut db.conn().unwrap(), new_id).unwrap();
    assert_eq!(
        list(&db.conn().unwrap(), bid).unwrap().len(),
        0,
        "expected empty list after delete"
    );
}

#[test]
fn delete_unknown_id_is_not_an_error() {
    let (db, _bid) = fresh();
    // Deleting a non-existent id should succeed (DELETE matches 0 rows).
    delete(&mut db.conn().unwrap(), 999_999).unwrap();
}
