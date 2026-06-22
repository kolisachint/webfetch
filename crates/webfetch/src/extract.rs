use scraper::{ElementRef, Html, Selector};

use crate::types::Metadata;

/// Pick the element most likely to contain the primary article content.
///
/// Heuristic, in priority order: `<article>`, `<main>`, `[role=main]`,
/// then the largest `<div>` by text length, falling back to `<body>`.
pub fn content_root(doc: &Html) -> Option<ElementRef<'_>> {
    for sel in ["article", "main", "[role=main]"] {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(el) = doc.select(&selector).next() {
                return Some(el);
            }
        }
    }

    // Fall back to the largest text-bearing <div>.
    if let Ok(div_sel) = Selector::parse("div") {
        let mut best: Option<(usize, ElementRef)> = None;
        for el in doc.select(&div_sel) {
            let len = el.text().map(|t| t.trim().len()).sum::<usize>();
            if best.as_ref().is_none_or(|(b, _)| len > *b) {
                best = Some((len, el));
            }
        }
        if let Some((len, el)) = best {
            if len > 0 {
                return Some(el);
            }
        }
    }

    Selector::parse("body")
        .ok()
        .and_then(|sel| doc.select(&sel).next())
}

/// Extract the page title from `<title>` or the first `<h1>`.
pub fn extract_title(doc: &Html) -> String {
    for sel in ["title", "h1"] {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(el) = doc.select(&selector).next() {
                let t = el.text().collect::<String>().trim().to_string();
                if !t.is_empty() {
                    return t;
                }
            }
        }
    }
    String::new()
}

/// Read the `content` attribute of the first matching `<meta>` selector.
fn meta(doc: &Html, selectors: &[&str]) -> Option<String> {
    for sel in selectors {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(el) = doc.select(&selector).next() {
                if let Some(c) = el.value().attr("content") {
                    let c = c.trim();
                    if !c.is_empty() {
                        return Some(c.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract citation-oriented metadata: description, author, publish date,
/// language, and site name (from standard `<meta>`/OpenGraph tags).
pub fn extract_metadata(doc: &Html) -> Metadata {
    let lang = Selector::parse("html")
        .ok()
        .and_then(|sel| doc.select(&sel).next())
        .and_then(|el| el.value().attr("lang"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Metadata {
        description: meta(
            doc,
            &["meta[name=description]", "meta[property='og:description']"],
        ),
        author: meta(
            doc,
            &["meta[name=author]", "meta[property='article:author']"],
        ),
        published: meta(
            doc,
            &[
                "meta[property='article:published_time']",
                "meta[name='date']",
            ],
        ),
        site_name: meta(doc, &["meta[property='og:site_name']"]),
        lang,
    }
}
