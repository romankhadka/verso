use verso::store::{
    books::{upsert as upsert_book, BookRow},
    db::Db,
    progress::{load, upsert, ProgressRow},
};

fn fresh() -> (Db, i64) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    let id = upsert_book(&mut db.conn().unwrap(), &BookRow::new_fixture("tm")).unwrap();
    std::mem::forget(tmp);
    (db, id)
}

#[test]
fn upsert_inserts_new_row() {
    let (db, bid) = fresh();
    let row = ProgressRow {
        book_id: bid,
        spine_idx: 0,
        char_offset: 1234,
        anchor_hash: "abc123".into(),
        percent: 12.5,
        time_read_s: 60,
        words_read: 0,
    };
    upsert(&mut db.conn().unwrap(), &row).unwrap();

    let got = load(&db.conn().unwrap(), bid).unwrap().unwrap();
    assert_eq!(got.book_id, bid);
    assert_eq!(got.spine_idx, 0);
    assert_eq!(got.char_offset, 1234);
    assert_eq!(got.anchor_hash, "abc123");
    assert!((got.percent - 12.5).abs() < f32::EPSILON);
    assert_eq!(got.time_read_s, 60);
    assert_eq!(got.words_read, 0);
}

#[test]
fn upsert_updates_existing() {
    let (db, bid) = fresh();
    let first = ProgressRow {
        book_id: bid,
        spine_idx: 0,
        char_offset: 100,
        anchor_hash: "first".into(),
        percent: 5.0,
        time_read_s: 30,
        words_read: 0,
    };
    upsert(&mut db.conn().unwrap(), &first).unwrap();

    let second = ProgressRow {
        book_id: bid,
        spine_idx: 1,
        char_offset: 9000,
        anchor_hash: "second".into(),
        percent: 42.0,
        time_read_s: 600,
        words_read: 0,
    };
    upsert(&mut db.conn().unwrap(), &second).unwrap();

    let count: i64 = db
        .conn()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM progress WHERE book_id = ?",
            rusqlite::params![bid],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    let got = load(&db.conn().unwrap(), bid).unwrap().unwrap();
    assert_eq!(got.spine_idx, 1);
    assert_eq!(got.char_offset, 9000);
    assert_eq!(got.anchor_hash, "second");
    assert!((got.percent - 42.0).abs() < f32::EPSILON);
    assert_eq!(got.time_read_s, 600);
}

#[test]
fn load_returns_none_when_missing() {
    let (db, _bid) = fresh();
    let got = load(&db.conn().unwrap(), 9_999_999).unwrap();
    assert!(got.is_none());
}
