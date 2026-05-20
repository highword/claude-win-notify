use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(local_app_data)
        .join("claude-win-notify")
        .join("logs")
}

pub fn log_error(msg: &str) {
    let dir = log_dir();
    let _ = fs::create_dir_all(&dir);
    let log_file = dir.join("error.log");

    rotate_if_needed(&log_file);

    let timestamp = format_timestamp();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_file) {
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

fn rotate_if_needed(log_file: &std::path::Path) {
    let metadata = match fs::metadata(log_file) {
        Ok(m) => m,
        Err(_) => return,
    };

    if metadata.len() > 1_048_576 {
        if let Ok(content) = fs::read(log_file) {
            let start = content.len().saturating_sub(524_288);
            let _ = fs::write(log_file, &content[start..]);
        }
    }
}

fn format_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Simple date calculation from days since epoch
    let mut y = 1970;
    let mut remaining_days = days as i64;

    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let month_days = if is_leap_year(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    for (i, &d) in month_days.iter().enumerate() {
        if remaining_days < d as i64 {
            month = i + 1;
            break;
        }
        remaining_days -= d as i64;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
