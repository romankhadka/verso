use std::path::Path;
use verso::config::{load, Config};

#[test]
fn defaults_are_sensible() {
    let cfg = Config::default();
    assert_eq!(cfg.reader.column_width, 68);
    assert_eq!(cfg.reader.theme, "dark");
    assert_eq!(cfg.reader.wpm, 250);
    assert!(cfg.library.watch);
}

#[test]
fn user_overrides_apply() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), r#"
[reader]
column_width = 80
theme = "sepia"
"#).unwrap();
    let cfg = load::from_path(tmp.path()).unwrap();
    assert_eq!(cfg.reader.column_width, 80);
    assert_eq!(cfg.reader.theme, "sepia");
    assert_eq!(cfg.reader.wpm, 250); // untouched default
}

#[test]
fn missing_file_returns_defaults() {
    let cfg = load::from_path(Path::new("/definitely/does/not/exist.toml")).unwrap();
    assert_eq!(cfg, Config::default());
}
