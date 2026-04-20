use ammonia::Builder;
use std::collections::HashSet;

/// Strip dangerous tags and attributes from EPUB HTML before any rendering.
pub fn clean(html: &str) -> String {
    let mut allowed_tags: HashSet<&str> = HashSet::new();
    for t in [
        "a","p","br","span","div","em","strong","b","i","u","s","small","sup","sub",
        "h1","h2","h3","h4","h5","h6","blockquote","cite","q","code","pre","kbd","samp",
        "ul","ol","li","dl","dt","dd","hr","img","figure","figcaption",
        "table","thead","tbody","tfoot","tr","th","td",
    ] { allowed_tags.insert(t); }

    Builder::default()
        .tags(allowed_tags)
        .strip_comments(true)
        .link_rel(Some("noopener noreferrer"))
        .clean(html)
        .to_string()
}
