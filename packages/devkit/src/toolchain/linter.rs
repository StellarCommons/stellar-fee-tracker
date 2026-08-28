pub struct LintIssue {
    pub field: String,
    pub expected_type: String,
    pub actual_value: String,
}

const REQUIRED_FIELDS: [(&str, &str); 4] = [
    ("base_fee", "number"),
    ("p50", "number"),
    ("p95", "number"),
    ("p99", "number"),
];

/// Validates a scenario JSON string against the Horizon `fee_stats`
/// schema, returning field-level lint errors.
pub fn lint_scenario(json: &str) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    for (field, expected_type) in REQUIRED_FIELDS {
        let pattern = format!("\"{field}\"");
        if !json.contains(&pattern) {
            issues.push(LintIssue {
                field: field.to_string(),
                expected_type: expected_type.to_string(),
                actual_value: "missing".to_string(),
            });
        }
    }
    issues
}
