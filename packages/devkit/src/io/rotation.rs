pub fn rotate_log_file_win(path: &str) -> Result<(), std::io::Error> {
    let sanitized = super::path_utils::sanitize_windows_path(path);
    if std::path::Path::new(&sanitized).exists() {
        std::fs::rename(&sanitized, format!("{}.old", sanitized))?;
    }
    Ok(())
}
