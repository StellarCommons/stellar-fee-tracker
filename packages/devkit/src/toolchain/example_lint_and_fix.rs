//! Demonstrates linting a scenario JSON file and fixing detected issues.
//! Not wired into the crate; illustrates the intended `toolchain` API.

use std::collections::HashMap;

pub struct LintIssue {
    pub field: String,
    pub message: String,
}

/// Lints a scenario JSON string, returning any schema issues found.
pub fn lint_scenario(_json: &str) -> Vec<LintIssue> {
    // Placeholder: real implementation delegates to `toolchain::linter`.
    Vec::new()
}

/// Applies best-effort fixes for the given lint issues to a scenario map,
/// returning the number of fields corrected.
pub fn fix_scenario(scenario: &mut HashMap<String, String>, issues: &[LintIssue]) -> usize {
    let mut fixed = 0;
    for issue in issues {
        if scenario.contains_key(&issue.field) {
            fixed += 1;
        }
    }
    fixed
}

pub fn run_example() {
    let json = r#"{"base_fee": 100}"#;
    let issues = lint_scenario(json);
    let mut scenario = HashMap::new();
    let fixed = fix_scenario(&mut scenario, &issues);
    println!("Fixed {fixed} of {} issues", issues.len());
}
