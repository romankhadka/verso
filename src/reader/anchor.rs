use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "s")]
    pub spine_idx: u32,
    #[serde(rename = "o")]
    pub char_offset: u64,
    #[serde(rename = "h")]
    pub anchor_hash: String,
}

/// 16-hex-char SHA-256 of the 50-char window centred on `char_offset` (by char, not byte).
///
/// The offset is quantised to 16-char buckets so that small drifts (< 16 chars) hash
/// to the same window — making the anchor hash stable across trivial re-imports.
pub fn anchor_hash(text: &str, char_offset: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let half = 25usize;
    // Quantise to 16-char buckets so tiny shifts produce the same window.
    let bucket = (char_offset / 16) * 16;
    let start = bucket.saturating_sub(half);
    let end = (bucket + half).min(chars.len());
    let window: String = chars[start..end].iter().collect();
    let digest = Sha256::digest(window.as_bytes());
    hex::encode(&digest[..8])
}

/// Try to relocate the passage `text` in `new_plaintext` given the original offset
/// and the stored context windows. Returns the new char-offset where `text` starts.
pub fn reanchor(
    new_plaintext: &str,
    text: &str,
    original_offset: usize,
    ctx_before: &str,
    ctx_after: &str,
) -> Option<usize> {
    // Strategy: search for ctx_before + text + ctx_after as a flexible window.
    let needle = format!("{ctx_before}{text}{ctx_after}");
    if let Some(i) = new_plaintext.find(&needle) {
        // Map byte index → char index, then offset by ctx_before length in chars.
        let char_i = new_plaintext[..i].chars().count();
        return Some(char_i + ctx_before.chars().count());
    }
    // Fallback: search for `text` alone; accept only if unique.
    let matches: Vec<usize> = new_plaintext.match_indices(text).map(|(i, _)| i).collect();
    if matches.len() == 1 {
        return Some(new_plaintext[..matches[0]].chars().count());
    }
    // Ambiguous: pick the match closest to the original offset.
    if !matches.is_empty() {
        let best = matches
            .into_iter()
            .map(|b| new_plaintext[..b].chars().count())
            .min_by_key(|c| (c.wrapping_sub(original_offset) as i64).abs())?;
        return Some(best);
    }
    None
}
