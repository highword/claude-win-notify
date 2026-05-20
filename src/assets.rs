use std::fs;
use std::path::{Path, PathBuf};

use crate::notification::NotificationType;

const HERO_TASK_COMPLETE: &[u8] = include_bytes!("../assets/hero-task-complete.png");
const HERO_PERMISSION: &[u8] = include_bytes!("../assets/hero-permission.png");
const HERO_QUESTION: &[u8] = include_bytes!("../assets/hero-question.png");
const HERO_ERROR: &[u8] = include_bytes!("../assets/hero-error.png");

/// Returns the assets directory path: `%LOCALAPPDATA%\claude-win-notify\assets`
pub fn assets_dir() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(local_app_data)
        .join("claude-win-notify")
        .join("assets")
}

/// Ensure the hero image for the given notification type exists in the specified directory.
///
/// This is the core logic function that extracts the embedded PNG to the given directory.
/// Returns the file path on success, or None on IO error (graceful degradation per D-09).
pub fn ensure_hero_image_in(dir: &Path, ntype: NotificationType) -> Option<PathBuf> {
    let path = dir.join(ntype.hero_filename());

    if path.exists() {
        return Some(path);
    }

    if let Err(e) = fs::create_dir_all(dir) {
        crate::log::log_error(&format!("Failed to create assets dir: {}", e));
        return None;
    }

    let data = hero_data(ntype);
    if let Err(e) = fs::write(&path, data) {
        crate::log::log_error(&format!("Failed to write hero image {}: {}", ntype.hero_filename(), e));
        return None;
    }

    Some(path)
}

/// Ensure the hero image for the given notification type exists on disk.
///
/// Extracts the embedded PNG to `%LOCALAPPDATA%\claude-win-notify\assets\` on first run.
/// Returns the file path on success, or None on IO error (graceful degradation per D-09).
pub fn ensure_hero_image(ntype: NotificationType) -> Option<PathBuf> {
    ensure_hero_image_in(&assets_dir(), ntype)
}

/// Get the embedded PNG data for a notification type.
fn hero_data(ntype: NotificationType) -> &'static [u8] {
    match ntype {
        NotificationType::TaskComplete => HERO_TASK_COMPLETE,
        NotificationType::PermissionRequest => HERO_PERMISSION,
        NotificationType::Question => HERO_QUESTION,
        NotificationType::Error => HERO_ERROR,
    }
}
