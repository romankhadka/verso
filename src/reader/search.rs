#[derive(Debug, Clone, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

pub fn find_matches(text: &str, needle: &str, _dir: SearchDirection) -> Vec<usize> {
    if needle.is_empty() {
        return vec![];
    }
    let hay = text.to_lowercase();
    let nee = needle.to_lowercase();
    hay.match_indices(&nee)
        .map(|(i, _)| text[..i].chars().count())
        .collect()
}
