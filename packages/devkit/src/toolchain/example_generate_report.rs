//! Demonstrates reading a fee CSV, running basic analysis, and generating
//! a Markdown report. Not wired into the crate; illustrates intended use
//! of `toolchain::reporter`.

pub struct CsvFeeRow {
    pub ledger_sequence: u64,
    pub fee: f64,
}

/// Parses a minimal `ledger_sequence,fee` CSV into fee rows.
pub fn parse_fee_csv(csv: &str) -> Vec<CsvFeeRow> {
    csv.lines()
        .skip(1) // header
        .filter_map(|line| {
            let mut parts = line.splitn(2, ',');
            let seq = parts.next()?.trim().parse().ok()?;
            let fee = parts.next()?.trim().parse().ok()?;
            Some(CsvFeeRow { ledger_sequence: seq, fee })
        })
        .collect()
}

pub fn run_example() {
    let csv = "ledger_sequence,fee\n100,250.0\n101,900.0\n";
    let rows = parse_fee_csv(csv);
    let avg = rows.iter().map(|r| r.fee).sum::<f64>() / rows.len().max(1) as f64;
    println!("## Fee Report\n\n- Records: {}\n- Average fee: {avg:.2}\n", rows.len());
}
