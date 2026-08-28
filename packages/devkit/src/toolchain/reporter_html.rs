pub struct FeeSummary {
    pub count: usize,
    pub avg_fee: f64,
    pub max_fee: f64,
    pub spike_count: usize,
    pub data_quality_score: f64,
}

/// Renders a self-contained HTML report: summary stats table, inline SVG
/// chart placeholder, spike list, and a data-quality score.
pub fn render_html_report(summary: &FeeSummary, chart_svg: &str, spikes: &[u64]) -> String {
    let spike_rows: String = spikes
        .iter()
        .map(|s| format!("<li>Ledger {s}</li>"))
        .collect();
    format!(
        "<html><body>\
         <h1>Fee Report</h1>\
         <table><tr><td>Count</td><td>{}</td></tr>\
         <tr><td>Avg Fee</td><td>{:.2}</td></tr>\
         <tr><td>Max Fee</td><td>{:.2}</td></tr>\
         <tr><td>Data Quality</td><td>{:.1}%</td></tr></table>\
         <div>{chart_svg}</div>\
         <ul>{spike_rows}</ul>\
         </body></html>",
        summary.count, summary.avg_fee, summary.max_fee, summary.data_quality_score * 100.0
    )
}
