use verso::library::hashing::sha256_file;

#[test]
fn hashes_time_machine_stably() {
    let h1 = sha256_file(std::path::Path::new("tests/fixtures/time-machine.epub")).unwrap();
    let h2 = sha256_file(std::path::Path::new("tests/fixtures/time-machine.epub")).unwrap();
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}
