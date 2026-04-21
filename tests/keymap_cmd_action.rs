use verso::ui::keymap::{
    defaults,
    table::{Dispatch, Keymap},
    Action,
};

#[test]
fn default_keymap_binds_colon_to_begin_cmd() {
    let entries = defaults::default_entries();
    let km = Keymap::from_config(&entries).unwrap();
    assert!(
        matches!(km.feed(":"), Dispatch::Fire(Action::BeginCmd)),
        "expected `:` to dispatch BeginCmd from the default keymap"
    );
}

#[test]
fn cmd_action_parses_from_config_string() {
    let km = Keymap::from_config(&[("cmd".into(), vec![":".into()])]).unwrap();
    assert!(matches!(km.feed(":"), Dispatch::Fire(Action::BeginCmd)));
}
