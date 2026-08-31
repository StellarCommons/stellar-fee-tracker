use super::sampler::CpuSample;
use std::time::Duration;

pub struct MultiSampleAggregator {
    pub samples: Vec<CpuSample>,
}

impl MultiSampleAggregator {
    pub fn new() -> Self {
        Self { samples: Vec::new() }
    }

    pub fn add(&mut self, sample: CpuSample) {
        self.samples.push(sample);
    }

    pub fn average_wall_time(&self) -> Duration {
        if self.samples.is_empty() { return Duration::from_secs(0); }
        let total: Duration = self.samples.iter().map(|s| s.wall_time).sum();
        total / self.samples.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_aggregator_average() {
        let mut agg = MultiSampleAggregator::new();
        agg.add(CpuSample::new("s1", Duration::from_millis(100), Duration::from_millis(100), 100.0));
        agg.add(CpuSample::new("s2", Duration::from_millis(200), Duration::from_millis(200), 100.0));
        assert_eq!(agg.average_wall_time(), Duration::from_millis(150));
    }
}
