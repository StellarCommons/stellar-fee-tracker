pub fn sanitize_windows_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sanitize_windows_path() {
        assert_eq!(sanitize_windows_path("C:\\logs\\file.log"), "C:/logs/file.log");
    }
}
