use std::time::Duration;
use verso::library::watch::{spawn_watcher, LibraryEvent};

#[test]
fn emits_create_event() {
    let tmp = tempfile::tempdir().unwrap();
    let (rx, _handle) = spawn_watcher(tmp.path()).unwrap();

    std::fs::write(tmp.path().join("a.epub"), b"stub").unwrap();

    let ev = rx.recv_timeout(Duration::from_secs(3)).expect("no event");
    assert!(matches!(
        ev,
        LibraryEvent::Created(_) | LibraryEvent::Changed
    ));
}
