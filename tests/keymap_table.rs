use verso::ui::keymap::{
    table::{Dispatch, Keymap},
    Action,
};

#[test]
fn dispatches_single_key_immediately() {
    let km = Keymap::from_config(&[("move_down".into(), vec!["j".into()])]).unwrap();
    let d1 = km.feed("j");
    assert!(matches!(d1, Dispatch::Fire(Action::MoveDown)));
}

#[test]
fn dispatches_chord_after_full_sequence() {
    let km = Keymap::from_config(&[("goto_top".into(), vec!["gg".into()])]).unwrap();
    assert!(matches!(km.feed("g"), Dispatch::Pending));
    assert!(matches!(km.feed("g"), Dispatch::Fire(Action::GotoTop)));
}

#[test]
fn rejects_prefix_collision() {
    let err = verso::ui::keymap::table::Keymap::from_config(&[
        ("move_down".into(), vec!["g".into()]),
        ("goto_top".into(), vec!["gg".into()]),
    ])
    .unwrap_err();
    assert!(err.to_string().contains("prefix"));
}
