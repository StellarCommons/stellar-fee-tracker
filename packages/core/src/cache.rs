use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ResponseCache<T: Clone> {
    value: Option<T>,
    cached_at: Option<Instant>,
    ttl: Duration,
}

impl<T: Clone> ResponseCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            value: None,
            cached_at: None,
            ttl,
        }
    }

    pub fn get(&self) -> Option<T> {
        if self.is_fresh() {
            return self.value.clone();
        }
        None
    }

    pub fn set(&mut self, value: T) {
        self.value = Some(value);
        self.cached_at = Some(Instant::now());
    }

    pub fn invalidate(&mut self) {
        self.value = None;
        self.cached_at = None;
    }

    pub fn is_fresh(&self) -> bool {
        match self.cached_at {
            Some(cached_at) => cached_at.elapsed() <= self.ttl,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn get_returns_none_when_empty() {
        let cache = ResponseCache::<u64>::new(Duration::from_secs(1));
        assert!(cache.get().is_none());
        assert!(!cache.is_fresh());
    }

    #[test]
    fn get_returns_value_when_fresh() {
        let mut cache = ResponseCache::new(Duration::from_secs(1));
        cache.set(42_u64);
        assert_eq!(cache.get(), Some(42));
        assert!(cache.is_fresh());
    }

    #[test]
    fn get_returns_none_when_expired() {
        let mut cache = ResponseCache::new(Duration::from_millis(5));
        cache.set(42_u64);
        thread::sleep(Duration::from_millis(10));
        assert!(cache.get().is_none());
        assert!(!cache.is_fresh());
    }

    #[test]
    fn invalidate_clears_cached_value() {
        let mut cache = ResponseCache::new(Duration::from_secs(1));
        cache.set(42_u64);
        cache.invalidate();
        assert!(cache.get().is_none());
        assert!(!cache.is_fresh());
    }
}
