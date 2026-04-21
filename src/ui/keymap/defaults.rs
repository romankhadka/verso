use std::collections::BTreeMap;

/// Default key bindings for the reader, as (action, sequences) pairs.
pub fn default_entries() -> Vec<(String, Vec<String>)> {
    vec![
        ("move_down".into(), vec!["j".into(), "<Down>".into()]),
        ("move_up".into(), vec!["k".into(), "<Up>".into()]),
        (
            "page_down".into(),
            vec!["<Space>".into(), "f".into(), "<C-f>".into()],
        ),
        ("page_up".into(), vec!["b".into(), "<C-b>".into()]),
        ("half_page_down".into(), vec!["d".into(), "<C-d>".into()]),
        ("half_page_up".into(), vec!["u".into(), "<C-u>".into()]),
        ("goto_top".into(), vec!["gg".into()]),
        ("goto_bottom".into(), vec!["G".into()]),
        ("next_chapter".into(), vec!["]]".into()]),
        ("prev_chapter".into(), vec!["[[".into()]),
        ("mark_set".into(), vec!["m".into()]),
        ("mark_jump".into(), vec!["'".into()]),
        ("search_forward".into(), vec!["/".into()]),
        ("search_backward".into(), vec!["?".into()]),
        ("search_next".into(), vec!["n".into()]),
        ("search_prev".into(), vec!["N".into()]),
        ("visual_select".into(), vec!["v".into()]),
        ("yank_highlight".into(), vec!["y".into()]),
        ("list_highlights".into(), vec!["H".into()]),
        ("cmd".into(), vec![":".into()]),
        ("quit_to_library".into(), vec!["q".into()]),
        ("toggle_theme".into(), vec!["gt".into()]),
        ("cycle_width".into(), vec!["z=".into()]),
        ("help".into(), vec!["<F1>".into()]),
    ]
}

/// Merge user keymap overrides on top of defaults.
/// If `user` contains an action, it REPLACES the default bindings for that action.
/// Unknown actions are still passed through so `Keymap::from_config` can surface
/// a clear error rather than silently ignoring user input.
pub fn merge_with_user(user: &BTreeMap<String, Vec<String>>) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = default_entries();
    for (action, seqs) in user {
        if let Some(entry) = out.iter_mut().find(|(a, _)| a == action) {
            entry.1 = seqs.clone();
        } else {
            out.push((action.clone(), seqs.clone()));
        }
    }
    out
}
