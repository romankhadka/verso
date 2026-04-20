use verso::util::paths::Paths;

#[test]
fn paths_resolve_to_xdg_subdirs() {
    let tmp = tempfile::tempdir().unwrap();
    let p = Paths::for_root(tmp.path());

    assert!(p.data_dir().ends_with("verso"));
    assert!(p.config_dir().ends_with("verso"));
    assert!(p.state_dir().ends_with("verso"));
    assert_eq!(p.db_file(), p.data_dir().join("verso.db"));
    assert_eq!(p.config_file(), p.config_dir().join("config.toml"));
}
