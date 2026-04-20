use verso::reader::linebreak::wrap;

#[test]
fn wraps_plain_text_to_column() {
    let para = "The quick brown fox jumps over the lazy dog and many other obstacles besides.";
    let lines = wrap(para, 30);
    for l in &lines { assert!(l.chars().count() <= 30, "{l:?} > 30"); }
    assert!(lines.len() >= 3);
    assert_eq!(lines.join(" "), para);
}

#[test]
fn preserves_paragraph_breaks() {
    let input = "First paragraph here.\n\nSecond paragraph here.";
    let lines = wrap(input, 30);
    let joined = lines.join("\n");
    assert!(joined.contains("First paragraph here."));
    assert!(joined.contains("Second paragraph here."));
    let blanks = lines.iter().filter(|l| l.is_empty()).count();
    assert_eq!(blanks, 1);
}
