//! Unit tests for the scenario file linter (`toolchain::linter`).
//! Kept standalone until the `toolchain` module is wired into the crate.

use super::linter::{lint_scenario, LintIssue};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_scenario_passes_without_issues() {
        let valid = r#"{"base_fee": 100, "p50": 100, "p95": 300, "p99": 500}"#;
        assert!(lint_scenario(valid).is_empty());
    }

    #[test]
    fn missing_required_field_produces_lint_error() {
        let invalid = r#"{"p50": 100}"#;
        let issues: Vec<LintIssue> = lint_scenario(invalid);
        assert!(issues.iter().any(|i| i.field == "base_fee"));
    }

    #[test]
    fn wrong_type_produces_lint_error() {
        let invalid = r#"{"base_fee": "not-a-number"}"#;
        let issues = lint_scenario(invalid);
        assert!(!issues.is_empty());
    }
}
