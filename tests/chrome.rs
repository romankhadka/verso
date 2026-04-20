use std::time::{Duration, Instant};
use verso::ui::chrome::{Chrome, ChromeState};

#[test]
fn transitions_from_visible_to_idle() {
    let mut c = Chrome::new(Duration::from_millis(50));
    c.touch(Instant::now());
    assert_eq!(c.state(Instant::now()), ChromeState::Visible);
    let later = Instant::now() + Duration::from_millis(100);
    assert_eq!(c.state(later), ChromeState::Idle);
}
