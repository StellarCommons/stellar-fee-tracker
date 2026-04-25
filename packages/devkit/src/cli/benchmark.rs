/// Runs benchmarks against the fee tracker pipeline.
pub struct Benchmark;

impl Benchmark {
    /// Runs SMA, EMA, and WMA on spike data and prints a comparison table.
    pub fn compare_spike(fees: &[f64], window: usize, alpha: f64) {
        use crate::analysis::rolling_window::RollingWindow;

        let sma = RollingWindow::sma(fees, window);
        let ema = RollingWindow::ema(fees, alpha);
        let wma = RollingWindow::wma(fees, window);

        println!("{:<6} {:>12} {:>12} {:>12}", "idx", "SMA", "EMA", "WMA");
        let len = sma.len().min(ema.len()).min(wma.len());
        // EMA starts from index 0; SMA/WMA start from index (window-1)
        let offset = window - 1;
        for i in 0..len {
            println!(
                "{:<6} {:>12.4} {:>12.4} {:>12.4}",
                i + offset,
                sma[i],
                ema[i + offset],
                wma[i]
            );
        }
    }
}
