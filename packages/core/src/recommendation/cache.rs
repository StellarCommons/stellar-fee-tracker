use std::time::{Duration, Instant};

use super::types::RecommendResponse;

struct CacheEntry {
    value: RecommendResponse,
    expires_at: Instant,
}

pub struct RecommendationCache {
    entry: Option<CacheEntry>,
    ttl: Duration,
}

impl RecommendationCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entry: None,
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn get(&self) -> Option<&RecommendResponse> {
        self.entry.as_ref().and_then(|e| {
            if e.expires_at > Instant::now() {
                Some(&e.value)
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, value: RecommendResponse) {
        self.entry = Some(CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    pub fn invalidate(&mut self) {
        self.entry = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recommendation::types::RecommendResponse;

    fn make_response() -> RecommendResponse {
        RecommendResponse {
            recommended_fee: "100".to_string(),
            fee_in_stroops: 100,
            estimated_wait_ledgers: 1,
            confidence: 0.95,
            network_condition: "normal".to_string(),
            alternatives: vec![],
        }
    }

    #[test]
    fn miss_on_empty_cache() {
        let cache = RecommendationCache::new(10);
        assert!(cache.get().is_none());
    }

    #[test]
    fn hit_after_set() {
        let mut cache = RecommendationCache::new(10);
        cache.set(make_response());
        assert!(cache.get().is_some());
    }

    #[test]
    fn miss_after_invalidate() {
        let mut cache = RecommendationCache::new(10);
        cache.set(make_response());
        cache.invalidate();
        assert!(cache.get().is_none());
    }

    #[test]
    fn expired_entry_returns_none() {
        let mut cache = RecommendationCache::new(0);
        cache.set(make_response());
        // ttl=0 → already expired
        assert!(cache.get().is_none());
    }
}
