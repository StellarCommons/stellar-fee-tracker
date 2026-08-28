pub struct ScenarioSummary {
    pub name: &'static str,
    pub description: &'static str,
    pub p50: u64,
    pub p95: u64,
}

/// Returns the bundled scenario library with key percentile metrics for
/// display in `devkit` CLI output.
pub fn list_scenarios() -> Vec<ScenarioSummary> {
    vec![
        ScenarioSummary {
            name: "normal",
            description: "Baseline low-fee environment",
            p50: 100,
            p95: 300,
        },
        ScenarioSummary {
            name: "congested",
            description: "High-load spike environment",
            p50: 45_000,
            p95: 150_000,
        },
    ]
}

/// Formats the scenario list as an aligned table for terminal output.
pub fn format_scenario_table(scenarios: &[ScenarioSummary]) -> String {
    let mut out = format!("{:<12} {:<24} {:>8} {:>8}\n", "Name", "Description", "p50", "p95");
    for s in scenarios {
        out.push_str(&format!(
            "{:<12} {:<24} {:>8} {:>8}\n",
            s.name, s.description, s.p50, s.p95
        ));
    }
    out
}
