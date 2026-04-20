use crossbeam_channel::{unbounded, Receiver};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum LibraryEvent {
    Created(PathBuf),
    Removed(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
    Changed,
}

/// Returns a receiver of library events and the watcher handle that must be kept alive.
pub fn spawn_watcher(
    dir: &Path,
) -> anyhow::Result<(Receiver<LibraryEvent>, notify::RecommendedWatcher)> {
    let (raw_tx, raw_rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: notify::RecommendedWatcher = notify::recommended_watcher(move |res| {
        let _ = raw_tx.send(res);
    })?;
    watcher.watch(dir, RecursiveMode::Recursive)?;

    let (out_tx, out_rx) = unbounded::<LibraryEvent>();
    std::thread::Builder::new()
        .name("verso-fs-watch".into())
        .spawn(move || {
            // 500 ms coalescing.
            let mut last_flush = Instant::now();
            let mut pending: Vec<LibraryEvent> = Vec::new();
            loop {
                match raw_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(Ok(ev)) => pending.extend(map_event(ev)),
                    Ok(Err(_e)) => {}
                    Err(_) => {}
                }
                if last_flush.elapsed() >= Duration::from_millis(500) && !pending.is_empty() {
                    for ev in pending.drain(..) {
                        if out_tx.send(ev).is_err() {
                            return;
                        }
                    }
                    last_flush = Instant::now();
                }
            }
        })?;

    Ok((out_rx, watcher))
}

fn map_event(ev: Event) -> Vec<LibraryEvent> {
    use EventKind::*;
    match ev.kind {
        Create(_) => ev.paths.into_iter().map(LibraryEvent::Created).collect(),
        Remove(_) => ev.paths.into_iter().map(LibraryEvent::Removed).collect(),
        Modify(notify::event::ModifyKind::Name(_)) if ev.paths.len() == 2 => {
            vec![LibraryEvent::Renamed {
                from: ev.paths[0].clone(),
                to: ev.paths[1].clone(),
            }]
        }
        _ => vec![LibraryEvent::Changed],
    }
}
