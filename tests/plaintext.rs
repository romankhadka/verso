use verso::reader::{sanitize, plaintext};

#[test]
fn strips_scripts_and_iframes() {
    let html = r#"<p>hi</p><script>alert(1)</script><iframe src="x"></iframe>"#;
    let safe = sanitize::clean(html);
    assert!(!safe.contains("<script"));
    assert!(!safe.contains("<iframe"));
    assert!(safe.contains("<p>hi</p>"));
}

#[test]
fn plaintext_normalises_whitespace() {
    let html = "<p>Hello,   world.</p>\n<p>Second\n\nline.</p>";
    let pt = plaintext::from_html(html);
    assert_eq!(pt, "Hello, world.\n\nSecond line.");
}

#[test]
fn plaintext_ignores_scripts() {
    let html = "<p>keep</p><script>drop</script>";
    let pt = plaintext::from_html(html);
    assert_eq!(pt, "keep");
}
