use verso::library::epub_guard::{validate_archive, Limits, GuardError};

#[test]
fn time_machine_passes_guards() {
    validate_archive(std::path::Path::new("tests/fixtures/time-machine.epub"), Limits::default()).unwrap();
}

#[test]
fn rejects_path_traversal() {
    // Build a tiny ZIP with a ../foo entry.
    let tmp = tempfile::NamedTempFile::new().unwrap();
    {
        let file = std::fs::File::create(tmp.path()).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("../evil.txt", zip::write::FileOptions::default()).unwrap();
        use std::io::Write;
        zip.write_all(b"nope").unwrap();
        zip.finish().unwrap();
    }
    let err = validate_archive(tmp.path(), Limits::default()).unwrap_err();
    assert!(matches!(err, GuardError::PathTraversal(_)));
}
