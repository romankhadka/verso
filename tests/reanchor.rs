use verso::reader::anchor::reanchor;

#[test]
fn drift_logic_marks_status_correctly() {
    let new_text = "Prelude paragraph added. Original content continues here exactly as before.";
    let highlight_text = "Original content continues here exactly";
    let original_offset = 0; // pre-import offset
    let hit = reanchor(new_text, highlight_text, original_offset, "paragraph added. ", " as before.");
    assert!(hit.is_some());
}
