use verso::reader::anchor::{Location, anchor_hash, reanchor};

#[test]
fn location_serializes_round_trip() {
    let loc = Location { spine_idx: 3, char_offset: 1842, anchor_hash: "abc123".into() };
    let json = serde_json::to_string(&loc).unwrap();
    let back: Location = serde_json::from_str(&json).unwrap();
    assert_eq!(loc, back);
}

#[test]
fn anchor_hash_is_stable_in_window() {
    let text = "hello world this is the anchor window being hashed";
    let h1 = anchor_hash(text, 20);
    let h2 = anchor_hash(text, 20);
    assert_eq!(h1, h2);
    let h3 = anchor_hash(text, 21);
    assert_eq!(h1, h3, "single-char drift should hash the same 50-char window");
}

#[test]
fn reanchor_finds_shifted_text() {
    let old = "AAAAA The cat sat on the mat BBBBB".to_string();
    let new = "AAAAA prefix paragraph. The cat sat on the mat BBBBB".to_string();
    let offset = old.find("The cat").unwrap();
    let result = reanchor(&new, "The cat sat on the mat", offset, "AAAAA ", " BBBBB");
    assert_eq!(result, Some(new.find("The cat").unwrap()));
}
