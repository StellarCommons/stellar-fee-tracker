//! Unit tests for the scenario file generator (`toolchain::generator`).
//! Kept standalone until the `toolchain` module is wired into the crate.

use super::generator::ScenarioGenerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_output_is_valid_json() {
        let json = ScenarioGenerator::new().base_fee(100).p50(200).generate();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn specified_fields_appear_with_correct_values() {
        let json = ScenarioGenerator::new()
            .base_fee(100)
            .p50(3849)
            .p95(61684)
            .p99(219192)
            .generate();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["base_fee"], 100);
        assert_eq!(value["p50"], 3849);
        assert_eq!(value["p95"], 61684);
        assert_eq!(value["p99"], 219192);
    }
}
