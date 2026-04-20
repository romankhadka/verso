use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub library: Library,
    pub reader: Reader,
    /// action -> one or more key sequences
    pub keymap: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Library {
    pub path: String,
    pub export_subdir: String,
    pub watch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Reader {
    pub column_width: u16,
    pub theme: String,
    pub chrome: String,
    pub chrome_idle_ms: u64,
    pub wpm: u32,
    pub code_wrap: String,
    pub min_term_cols: u16,
    pub min_term_rows: u16,
}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            library: Library::default(),
            reader: Reader::default(),
            keymap: BTreeMap::new(),
        }
    }
}
impl Default for Library {
    fn default() -> Self {
        Self {
            path: "~/Books".into(),
            export_subdir: "highlights".into(),
            watch: true,
        }
    }
}
impl Default for Reader {
    fn default() -> Self {
        Self {
            column_width: 68,
            theme: "dark".into(),
            chrome: "autohide".into(),
            chrome_idle_ms: 3000,
            wpm: 250,
            code_wrap: "scroll".into(),
            min_term_cols: 40,
            min_term_rows: 10,
        }
    }
}
