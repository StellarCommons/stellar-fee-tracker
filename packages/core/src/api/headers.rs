use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use axum::http::{HeaderMap, HeaderValue, header};
use chrono::{DateTime, Utc};

/// Compute a weakly-stable quoted ETag from response bytes.
pub fn compute_etag(body: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    body.hash(&mut hasher);
    format!("\"{:x}\"", hasher.finish())
}

/// Build a Cache-Control value using max-age and stale-while-revalidate.
pub fn cache_control(max_age: u32, swr: u32) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "max-age={}, stale-while-revalidate={}",
        max_age, swr
    ))
    .expect("cache-control header value should be valid")
}

/// Build an RFC 7231 HTTP-date for Last-Modified.
pub fn last_modified(timestamp: DateTime<Utc>) -> HeaderValue {
    HeaderValue::from_str(&timestamp.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
        .expect("last-modified header value should be valid")
}

/// Returns true when `If-None-Match` contains `*` or the exact current ETag.
pub fn if_none_match_matches(headers: &HeaderMap, current_etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(|raw| {
            raw.split(',')
                .map(|tag| tag.trim())
                .any(|tag| tag == "*" || tag == current_etag)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etag_is_quoted() {
        let etag = compute_etag(br#"{"ok":true}"#);
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
    }

    #[test]
    fn if_none_match_matches_exact_tag() {
        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, HeaderValue::from_static("\"abc\""));

        assert!(if_none_match_matches(&headers, "\"abc\""));
        assert!(!if_none_match_matches(&headers, "\"def\""));
    }
}
