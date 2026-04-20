use verso::store::{db::Db, books::{BookRow, upsert}, highlights::{insert, list, Highlight, AnchorStatus}};

fn fresh() -> (Db, i64) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    let id = upsert(&mut db.conn().unwrap(), &BookRow::new_fixture("tm")).unwrap();
    std::mem::forget(tmp);
    (db, id)
}

#[test]
fn inserts_and_lists_highlights() {
    let (db, bid) = fresh();
    let h = Highlight {
        id: 0, book_id: bid, spine_idx: 1, chapter_title: Some("Ch.1".into()),
        char_offset_start: 100, char_offset_end: 110,
        text: "Hello hi".into(), context_before: Some("pre".into()), context_after: Some("post".into()),
        note: None, anchor_status: AnchorStatus::Ok,
    };
    insert(&mut db.conn().unwrap(), &h).unwrap();
    let all = list(&db.conn().unwrap(), bid).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].text, "Hello hi");
}
