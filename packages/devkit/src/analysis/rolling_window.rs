/// Maintains a rolling window of fee observations.
pub struct RollingWindow;

impl RollingWindow {
    /// Simple moving average over `window` elements.
    pub fn sma(fees: &[u64], window: usize) -> Vec<f64> {
        if window == 0 || fees.len() < window {
            return vec![];
        }
        let mut sum: u64 = fees[..window].iter().sum();
        let mut out = Vec::with_capacity(fees.len() - window + 1);
        out.push(sum as f64 / window as f64);
        for i in window..fees.len() {
            sum += fees[i];
            sum -= fees[i - window];
            out.push(sum as f64 / window as f64);
        }
        out
    }

    /// Exponential moving average with smoothing factor `alpha` (0 < alpha <= 1).
    pub fn ema(fees: &[u64], alpha: f64) -> Vec<f64> {
        if fees.is_empty() {
            return vec![];
        }
        let mut out = Vec::with_capacity(fees.len());
        let mut prev = fees[0] as f64;
        out.push(prev);
        for &v in &fees[1..] {
            prev = alpha * v as f64 + (1.0 - alpha) * prev;
            out.push(prev);
        }
        out
    }

    /// Weighted moving average over `window` elements (linear weights).
    pub fn wma(fees: &[u64], window: usize) -> Vec<f64> {
        if window == 0 || fees.len() < window {
            return vec![];
        }
        let denom = (window * (window + 1) / 2) as f64;
        let mut out = Vec::with_capacity(fees.len() - window + 1);
        for i in (window - 1)..fees.len() {
            let val: f64 = fees[(i + 1 - window)..=i]
                .iter()
                .enumerate()
                .map(|(j, &v)| (j + 1) as f64 * v as f64)
                .sum::<f64>()
                / denom;
            out.push(val);
        }
        out
    }
}
