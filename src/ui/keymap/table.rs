use super::{
    keys::{parse_sequence, Key},
    Action,
};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Dispatch {
    Fire(Action),
    Pending,
    Unbound,
}

#[derive(Debug)]
pub struct Keymap {
    rules: Vec<(Vec<Key>, Action)>,
    buffer: std::cell::RefCell<Vec<Key>>,
}

impl Keymap {
    pub fn from_config(entries: &[(String, Vec<String>)]) -> anyhow::Result<Self> {
        let mut rules: Vec<(Vec<Key>, Action)> = Vec::new();
        for (action_str, seqs) in entries {
            let action = Action::from_str(action_str).map_err(|e| anyhow::anyhow!(e))?;
            for s in seqs {
                rules.push((parse_sequence(s)?, action));
            }
        }
        // Prefix check: no sequence can be a strict prefix of another.
        for i in 0..rules.len() {
            for j in 0..rules.len() {
                if i == j {
                    continue;
                }
                let (a, _) = &rules[i];
                let (b, _) = &rules[j];
                if b.len() > a.len() && &b[..a.len()] == a.as_slice() {
                    return Err(anyhow::anyhow!(
                        "keymap prefix collision: {:?} is a prefix of {:?}",
                        a,
                        b
                    ));
                }
            }
        }
        Ok(Self {
            rules,
            buffer: std::cell::RefCell::new(Vec::new()),
        })
    }

    /// Feed one user keystroke. Returns what to do.
    pub fn feed(&self, raw: &str) -> Dispatch {
        let keys = parse_sequence(raw).unwrap_or_default();
        let mut buf = self.buffer.borrow_mut();
        buf.extend(keys);

        // Exact full match?
        for (seq, action) in &self.rules {
            if *seq == *buf {
                buf.clear();
                return Dispatch::Fire(*action);
            }
        }
        // Proper prefix of some rule?
        for (seq, _) in &self.rules {
            if seq.len() > buf.len() && seq.starts_with(buf.as_slice()) {
                return Dispatch::Pending;
            }
        }
        buf.clear();
        Dispatch::Unbound
    }
}
