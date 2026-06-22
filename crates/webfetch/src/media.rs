//! Decide how to treat a fetched body. The HTML extractor only makes sense
//! for HTML; running it over a JSON API response, a raw `.txt`, or a Markdown
//! file would mangle or drop the content. We classify by `Content-Type` when
//! present, and sniff the body otherwise.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Media {
    Html,
    Json,
    Text,
    /// Anything we don't render (binary, PDF, images): the label is the
    /// content-type, surfaced to the caller.
    Other(String),
}

impl Media {
    /// A short, stable label for the `media` field of a result.
    pub fn label(&self) -> String {
        match self {
            Media::Html => "html".into(),
            Media::Json => "json".into(),
            Media::Text => "text".into(),
            Media::Other(ct) => ct.clone(),
        }
    }
}

/// Classify a body using its `Content-Type` header if available, else by
/// sniffing the first non-whitespace bytes.
pub fn classify(content_type: Option<&str>, body: &str) -> Media {
    if let Some(ct) = content_type {
        let essence = ct
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if essence.contains("html") || essence == "application/xhtml+xml" {
            return Media::Html;
        }
        if essence.contains("json") {
            return Media::Json;
        }
        if essence.starts_with("text/") {
            return Media::Text;
        }
        if !essence.is_empty() {
            return Media::Other(essence);
        }
    }
    sniff(body)
}

fn sniff(body: &str) -> Media {
    let trimmed = body.trim_start();
    match trimmed.as_bytes().first() {
        Some(b'<') => Media::Html,
        Some(b'{') | Some(b'[') => {
            if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
                Media::Json
            } else {
                Media::Text
            }
        }
        _ => Media::Text,
    }
}
