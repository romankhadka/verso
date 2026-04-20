use verso::ui::keymap::{defaults, table::Keymap};

#[test]
fn defaults_load_without_conflicts() {
    let entries = defaults::default_entries();
    let _km = Keymap::from_config(&entries).expect("default keymap should load");
}
