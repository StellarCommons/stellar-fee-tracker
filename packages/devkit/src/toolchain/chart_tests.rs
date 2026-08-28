//! Unit tests for the SVG chart generator (`toolchain::chart`).
//! Kept standalone until the `toolchain` module is wired into the crate.

use super::chart::{render_svg_chart, ChartOptions, FeePoint};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<FeePoint> {
        vec![
            FeePoint { timestamp: 1, fee: 100.0, is_spike: false },
            FeePoint { timestamp: 2, fee: 900.0, is_spike: true },
        ]
    }

    #[test]
    fn output_is_valid_svg() {
        let svg = render_svg_chart(&sample_points(), &ChartOptions::default());
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn spike_marker_present_for_spike_records() {
        let svg = render_svg_chart(&sample_points(), &ChartOptions::default());
        assert!(svg.contains("#dc2626"));
    }
}
