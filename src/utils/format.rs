use chrono::{DateTime, Local, Utc};

/// Format a file size in bytes to a human-readable string.
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format a UTC datetime for display in the user's local timezone.
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M").to_string()
}

/// Format a relative time string (e.g. "3 minutes ago").
pub fn format_relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*dt);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        let m = diff.num_minutes();
        format!("{} min{} ago", m, if m == 1 { "" } else { "s" })
    } else if diff.num_hours() < 24 {
        let h = diff.num_hours();
        format!("{} hour{} ago", h, if h == 1 { "" } else { "s" })
    } else if diff.num_days() < 30 {
        let d = diff.num_days();
        format!("{} day{} ago", d, if d == 1 { "" } else { "s" })
    } else {
        format_datetime(dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
