/// Severity level of a detected fee spike.
#[derive(Debug, Clone, PartialEq)]
pub enum SpikeSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// A single detected fee spike event.
#[derive(Debug, Clone)]
pub struct SpikeEvent {
    pub timestamp: u64,
    pub fee_amount: u64,
    pub baseline: u64,
    pub multiplier: f64,
    pub severity: SpikeSeverity,
}

/// Classifies fee spikes in a time series.
pub struct SpikeClassifier;

impl SpikeClassifier {
    /// Detects spikes where `fee > baseline * threshold`.
    /// Returns a `Vec<SpikeEvent>` for every entry that exceeds the threshold.
    pub fn detect(fees: &[(u64, u64)], baseline: u64, threshold: f64) -> Vec<SpikeEvent> {
        fees.iter()
            .filter_map(|&(timestamp, fee_amount)| {
                let multiplier = fee_amount as f64 / baseline as f64;
                if multiplier > threshold {
                    let severity = if multiplier < 2.0 {
                        SpikeSeverity::Low
                    } else if multiplier < 5.0 {
                        SpikeSeverity::Medium
                    } else if multiplier < 10.0 {
                        SpikeSeverity::High
                    } else {
                        SpikeSeverity::Critical
                    };
                    Some(SpikeEvent { timestamp, fee_amount, baseline, multiplier, severity })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_spikes_above_threshold() {
        let fees = [(1, 100), (2, 250), (3, 90), (4, 600)];
        let spikes = SpikeClassifier::detect(&fees, 100, 1.5);
        assert_eq!(spikes.len(), 2);
        assert_eq!(spikes[0].timestamp, 2);
        assert_eq!(spikes[1].timestamp, 4);
    }

    #[test]
    fn no_spikes_below_threshold() {
        let fees = [(1, 100), (2, 110), (3, 95)];
        let spikes = SpikeClassifier::detect(&fees, 100, 1.5);
        assert!(spikes.is_empty());
    }

    #[test]
    fn severity_levels() {
        let fees = [(1, 150), (2, 300), (3, 700), (4, 1100)];
        let spikes = SpikeClassifier::detect(&fees, 100, 1.0);
        assert_eq!(spikes[0].severity, SpikeSeverity::Low);
        assert_eq!(spikes[1].severity, SpikeSeverity::Medium);
        assert_eq!(spikes[2].severity, SpikeSeverity::High);
        assert_eq!(spikes[3].severity, SpikeSeverity::Critical);
    }
}
