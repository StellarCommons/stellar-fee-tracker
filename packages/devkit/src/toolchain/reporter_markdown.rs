pub struct FeeRecord {
    pub ledger_sequence: u64,
    pub fee: f64,
}

/// Renders a Markdown report summarizing a fee sequence, suitable for
/// posting into a GitHub issue or PR description.
pub fn render_markdown_report(records: &[FeeRecord]) -> String {
    let count = records.len();
    let avg = if count == 0 {
        0.0
    } else {
        records.iter().map(|r| r.fee).sum::<f64>() / count as f64
    };
    let max = records.iter().map(|r| r.fee).fold(0.0_f64, f64::max);

    let mut out = String::from("## Fee Report\n\n");
    out.push_str(&format!("- Records: {count}\n"));
    out.push_str(&format!("- Average fee: {avg:.2}\n"));
    out.push_str(&format!("- Max fee: {max:.2}\n\n"));
    out.push_str("| Ledger | Fee |\n|---|---|\n");
    for r in records {
        out.push_str(&format!("| {} | {:.2} |\n", r.ledger_sequence, r.fee));
    }
    out
}
