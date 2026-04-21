use std::collections::BTreeMap;
use verso::ui::keymap::defaults::merge_with_user;

#[test]
fn user_overrides_replace_default_bindings() {
    let mut user = BTreeMap::new();
    user.insert(
        "move_down".to_string(),
        vec!["<Down>".to_string(), "J".to_string()],
    );
    let merged = merge_with_user(&user);
    let md = merged.iter().find(|(a, _)| a == "move_down").unwrap();
    assert_eq!(md.1, vec!["<Down>".to_string(), "J".to_string()]);
}

#[test]
fn actions_not_overridden_keep_defaults() {
    let user = BTreeMap::new();
    let merged = merge_with_user(&user);
    let gg = merged.iter().find(|(a, _)| a == "goto_top").unwrap();
    assert_eq!(gg.1, vec!["gg".to_string()]);
}
