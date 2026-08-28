/// A single point in a fee-over-time sequence.
pub struct FeePoint {
    pub timestamp: i64,
    pub fee: f64,
    pub is_spike: bool,
}

pub struct ChartOptions {
    pub width: u32,
    pub height: u32,
    pub color: &'static str,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self { width: 800, height: 300, color: "#2563eb" }
    }
}

/// Renders a dependency-free SVG line chart for a fee sequence, marking
/// spike events in red.
pub fn render_svg_chart(points: &[FeePoint], opts: &ChartOptions) -> String {
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">",
        opts.width, opts.height
    );
    for (i, p) in points.iter().enumerate() {
        let color = if p.is_spike { "#dc2626" } else { opts.color };
        let x = i as u32 * 4;
        svg.push_str(&format!(
            "<circle cx=\"{x}\" cy=\"{}\" r=\"2\" fill=\"{color}\" />",
            opts.height as f64 - p.fee.min(opts.height as f64)
        ));
    }
    svg.push_str("</svg>");
    svg
}
