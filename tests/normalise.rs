use verso::library::normalise::{normalise_author, normalise_text};

#[test]
fn collapses_whitespace_and_case() {
    assert_eq!(normalise_text("  The  Time  Machine! "), "the time machine");
}

#[test]
fn normalises_authors() {
    assert_eq!(normalise_author("H. G. Wells"), "h g wells");
    assert_eq!(normalise_author("Wells, H. G."), "wells h g");
}
