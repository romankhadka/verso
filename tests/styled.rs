use verso::reader::styled::{to_spans, Span, Style};

#[test]
fn extracts_spans_from_html() {
    let spans = to_spans("<p>Hello <em>world</em>, <strong>now</strong>.</p>");
    let txts: Vec<_> = spans.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(txts.join(""), "Hello world, now.");
    assert!(spans.iter().any(|s| s.text == "world" && s.style.italic));
    assert!(spans.iter().any(|s| s.text == "now" && s.style.bold));
}
