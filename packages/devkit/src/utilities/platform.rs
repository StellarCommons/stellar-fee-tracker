pub fn detect_platform() -> &'static str {
    #[cfg(target_os = "windows")] { "windows" }
    #[cfg(target_os = "macos")] { "macos" }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))] { "unix" }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_platform() {
        let p = detect_platform();
        assert!(!p.is_empty());
    }
}
