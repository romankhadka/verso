pub fn normalise_text(s: &str) -> String {
    let lower = s.to_lowercase();
    let stripped: String = lower.chars().map(|c|
        if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' }
    ).collect();
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn normalise_author(s: &str) -> String {
    normalise_text(s)
}
