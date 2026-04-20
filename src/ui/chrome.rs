use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChromeState { Visible, Idle }

pub struct Chrome {
    idle_after: Duration,
    last_input: Option<Instant>,
}

impl Chrome {
    pub fn new(idle_after: Duration) -> Self { Self { idle_after, last_input: None } }
    pub fn touch(&mut self, now: Instant) { self.last_input = Some(now); }
    pub fn state(&self, now: Instant) -> ChromeState {
        match self.last_input {
            Some(t) if now.saturating_duration_since(t) < self.idle_after => ChromeState::Visible,
            _ => ChromeState::Idle,
        }
    }
}
