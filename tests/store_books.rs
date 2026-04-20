use verso::store::{db::Db, books::{BookRow, upsert, resolve_identity, IdentityMatch}};

fn fresh_db() -> Db {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db = Db::open(tmp.path()).unwrap();
    db.migrate().unwrap();
    std::mem::forget(tmp); // keep the file alive for the duration of the test
    db
}

#[test]
fn inserts_new_book_by_stable_id() {
    let db = fresh_db();
    let mut c = db.conn().unwrap();
    let row = BookRow::new_fixture("tm");
    let id = upsert(&mut c, &row).unwrap();
    assert!(id > 0);
    let m = resolve_identity(&c, &row).unwrap();
    assert!(matches!(m, Some(IdentityMatch::ById(_))));
}

#[test]
fn updates_existing_on_stable_id_match_with_new_hash() {
    let db = fresh_db();
    let mut c = db.conn().unwrap();
    let mut row = BookRow::new_fixture("tm");
    let id1 = upsert(&mut c, &row).unwrap();
    row.file_hash = Some("newhashvalue".into());
    let id2 = upsert(&mut c, &row).unwrap();
    assert_eq!(id1, id2, "same stable_id must reuse id");
    let m = resolve_identity(&c, &row).unwrap();
    assert!(matches!(m, Some(IdentityMatch::ById(_))));
}

#[test]
fn resolves_by_norm_fallback_when_no_ids_match() {
    let db = fresh_db();
    let mut c = db.conn().unwrap();
    let mut row = BookRow::new_fixture("tm");
    row.stable_id = None;
    row.file_hash = Some("hashA".into());
    let id = upsert(&mut c, &row).unwrap();
    let candidate = BookRow { stable_id: None, file_hash: Some("hashB".into()), ..row.clone() };
    let m = resolve_identity(&c, &candidate).unwrap();
    assert!(matches!(m, Some(IdentityMatch::ByNorm(rid)) if rid == id));
}
