pub fn export_flamegraph_json(label: &str) -> String {
    format!("{{\"flamegraph\": \"{}\"}}", label)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_export_flamegraph() {
        assert!(export_flamegraph_json("test").contains("flamegraph"));
    }
}
