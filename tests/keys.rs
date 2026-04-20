use verso::ui::keymap::keys::{parse_sequence, Key};

#[test]
fn parses_single_chars_and_chords() {
    assert_eq!(parse_sequence("j").unwrap(), vec![Key::Char('j')]);
    assert_eq!(parse_sequence("gg").unwrap(), vec![Key::Char('g'), Key::Char('g')]);
    assert_eq!(parse_sequence("]]").unwrap(), vec![Key::Char(']'), Key::Char(']')]);
}

#[test]
fn parses_named_keys() {
    assert_eq!(parse_sequence("<Space>").unwrap(), vec![Key::Named("Space".into())]);
    assert_eq!(parse_sequence("<C-d>").unwrap(), vec![Key::CtrlChar('d')]);
    assert_eq!(parse_sequence("<Esc>").unwrap(), vec![Key::Named("Esc".into())]);
}
