#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    CtrlChar(char),
    Named(String), // "Space", "Enter", "Esc", "Up", "Down", …
}

pub fn parse_sequence(seq: &str) -> anyhow::Result<Vec<Key>> {
    let mut out = Vec::new();
    let chars: Vec<char> = seq.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            let end = chars[i..]
                .iter()
                .position(|&c| c == '>')
                .ok_or_else(|| anyhow::anyhow!("unterminated <...> in {seq}"))?
                + i;
            let tok: String = chars[i + 1..end].iter().collect();
            i = end + 1;
            if let Some(rest) = tok.strip_prefix("C-") {
                let ch = rest
                    .chars()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("bad ctrl in {seq}"))?;
                out.push(Key::CtrlChar(ch));
            } else {
                out.push(Key::Named(tok));
            }
        } else {
            out.push(Key::Char(chars[i]));
            i += 1;
        }
    }
    Ok(out)
}
