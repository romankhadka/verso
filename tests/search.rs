use verso::reader::search::{find_matches, SearchDirection};

#[test]
fn finds_case_insensitive_matches() {
    let text = "Foo bar foo Baz FOOBAR";
    let m = find_matches(text, "foo", SearchDirection::Forward);
    assert_eq!(m.len(), 3);
}
