use hyphenation::{Language, Load, Standard};
use textwrap::Options;

pub fn wrap(text: &str, width: u16) -> Vec<String> {
    let dict = Standard::from_embedded(Language::EnglishUS).ok();
    let mut opts = Options::new(width as usize).break_words(false);
    if let Some(d) = dict.as_ref() {
        opts = opts.word_splitter(textwrap::WordSplitter::Hyphenation(d.clone()));
    }

    let mut out = Vec::new();
    for (i, para) in text.split("\n\n").enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        let wrapped = textwrap::wrap(para, &opts);
        for line in wrapped {
            out.push(line.into_owned());
        }
    }
    out
}
