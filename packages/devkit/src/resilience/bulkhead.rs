use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkheadError {
    ZeroCapacity,
    Saturated { capacity: usize },
}

#[derive(Debug)]
pub struct Bulkhead {
    max_concurrent: usize,
    in_flight: AtomicUsize,
    peak_in_flight: AtomicU64,
    admitted: AtomicU64,
    rejected: AtomicU64,
}

impl Bulkhead {
    pub fn new(max_concurrent: usize) -> Result<Self, BulkheadError> {
        if max_concurrent == 0 {
            return Err(BulkheadError::ZeroCapacity);
        }
        Ok(Self {
            max_concurrent,
            in_flight: AtomicUsize::new(0),
            peak_in_flight: AtomicU64::new(0),
            admitted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
        })
    }

    pub fn capacity(&self) -> usize {
        self.max_concurrent
    }

    pub fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::Relaxed)
    }

    pub fn available(&self) -> usize {
        self.max_concurrent.saturating_sub(self.in_flight())
    }

    pub fn is_saturated(&self) -> bool {
        self.in_flight() >= self.max_concurrent
    }

    pub fn peak_in_flight(&self) -> usize {
        self.peak_in_flight.load(Ordering::Relaxed) as usize
    }

    pub fn admitted(&self) -> u64 {
        self.admitted.load(Ordering::Relaxed)
    }

    pub fn rejected(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }

    pub fn try_acquire(&self) -> Result<BulkheadPermit<'_>, BulkheadError> {
        let mut current = self.in_flight.load(Ordering::Relaxed);
        loop {
            if current >= self.max_concurrent {
                self.rejected.fetch_add(1, Ordering::Relaxed);
                return Err(BulkheadError::Saturated {
                    capacity: self.max_concurrent,
                });
            }
            match self.in_flight.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.admitted.fetch_add(1, Ordering::Relaxed);
                    self.peak_in_flight
                        .fetch_max(current as u64 + 1, Ordering::Relaxed);
                    return Ok(BulkheadPermit { bulkhead: self });
                }
                Err(observed) => current = observed,
            }
        }
    }

    pub fn execute<T, F: FnOnce() -> T>(&self, operation: F) -> Result<T, BulkheadError> {
        let permit = self.try_acquire()?;
        drop(permit);
        Ok(operation())
    }
}

#[derive(Debug)]
pub struct BulkheadPermit<'a> {
    bulkhead: &'a Bulkhead,
}

impl Drop for BulkheadPermit<'_> {
    fn drop(&mut self) {
        self.bulkhead.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn a_bulkhead_needs_a_capacity() {
        assert_eq!(Bulkhead::new(0).err(), Some(BulkheadError::ZeroCapacity));
    }

    #[test]
    fn a_fresh_bulkhead_is_empty() {
        let bulkhead = Bulkhead::new(2).expect("bulkhead");
        assert_eq!(bulkhead.in_flight(), 0);
        assert_eq!(bulkhead.available(), 2);
        assert!(!bulkhead.is_saturated());
    }

    #[test]
    fn acquiring_frees_a_slot() {
        let bulkhead = Bulkhead::new(2).expect("bulkhead");
        let first = bulkhead.try_acquire().expect("permit");
        assert_eq!(bulkhead.in_flight(), 1);
        assert_eq!(bulkhead.available(), 1);
        drop(first);
        assert_eq!(bulkhead.in_flight(), 0);
    }

    #[test]
    fn the_last_slot_can_be_taken() {
        let bulkhead = Bulkhead::new(1).expect("bulkhead");
        let permit = bulkhead.try_acquire().expect("permit");
        assert!(bulkhead.is_saturated());
        drop(permit);
    }

    #[test]
    fn exceeding_the_limit_is_rejected() {
        let bulkhead = Bulkhead::new(1).expect("bulkhead");
        let _permit = bulkhead.try_acquire().expect("permit");
        assert_eq!(
            bulkhead.try_acquire().err(),
            Some(BulkheadError::Saturated { capacity: 1 })
        );
    }

    #[test]
    fn a_rejected_call_is_counted() {
        let bulkhead = Bulkhead::new(1).expect("bulkhead");
        let _permit = bulkhead.try_acquire().expect("permit");
        let _ = bulkhead.try_acquire();
        let _ = bulkhead.try_acquire();
        assert_eq!(bulkhead.rejected(), 2);
        assert_eq!(bulkhead.admitted(), 1);
    }

    #[test]
    fn the_peak_is_recorded() {
        let bulkhead = Bulkhead::new(3).expect("bulkhead");
        let first = bulkhead.try_acquire().expect("permit");
        let second = bulkhead.try_acquire().expect("permit");
        assert_eq!(bulkhead.peak_in_flight(), 2);
        drop(first);
        drop(second);
        assert_eq!(bulkhead.peak_in_flight(), 2);
    }

    #[test]
    fn execute_runs_the_operation() {
        let bulkhead = Bulkhead::new(1).expect("bulkhead");
        assert_eq!(bulkhead.execute(|| 42), Ok(42));
        assert_eq!(bulkhead.in_flight(), 0);
    }

    #[test]
    fn execute_refuses_when_saturated() {
        let bulkhead = Bulkhead::new(1).expect("bulkhead");
        let _permit = bulkhead.try_acquire().expect("permit");
        assert!(bulkhead.execute(|| 42).is_err());
    }

    #[test]
    fn a_saturated_bulkhead_recovers_after_release() {
        let bulkhead = Bulkhead::new(1).expect("bulkhead");
        let permit = bulkhead.try_acquire().expect("permit");
        assert!(bulkhead.try_acquire().is_err());
        drop(permit);
        assert!(bulkhead.try_acquire().is_ok());
    }

    #[test]
    fn concurrent_callers_never_exceed_the_limit() {
        let bulkhead = Arc::new(Bulkhead::new(4).expect("bulkhead"));
        let mut handles = Vec::new();
        for _ in 0..16 {
            let bulkhead = Arc::clone(&bulkhead);
            handles.push(thread::spawn(move || {
                let permit = bulkhead.try_acquire();
                if permit.is_ok() {
                    thread::yield_now();
                }
                permit.is_ok()
            }));
        }
        let mut admitted = 0;
        let mut rejected_by_thread = 0;
        for handle in handles {
            if handle.join().expect("thread") {
                admitted += 1;
            } else {
                rejected_by_thread += 1;
            }
        }
        assert!(admitted > 0);
        assert!(bulkhead.peak_in_flight() <= 4);
        assert_eq!(bulkhead.in_flight(), 0);
        assert_eq!(rejected_by_thread, 16 - admitted);
        assert_eq!(bulkhead.admitted(), admitted as u64);
        assert_eq!(bulkhead.admitted() + bulkhead.rejected(), 16);
    }
}
