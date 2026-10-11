//! File paths and environment directories for Kora config and assets

use std::fs;
use std::path::PathBuf;

/// Returns the absolute path to ~/.config/kora/config.ini
pub fn get_config_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&home)
        .join(".config")
        .join("kora")
        .join("config.ini")
}

/// Returns the path to the directory containing switch soundpacks if it exists.
/// Checks ~/.config/kora/switches, ./switches.
pub fn get_switches_dir() -> Option<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let kora_config = PathBuf::from(&home).join(".config").join("kora");
    let candidates = [
        kora_config.join("switches"),
        kora_config.join("switchs"),
        PathBuf::from("switches"),
        PathBuf::from("switchs"),
    ];
    candidates.into_iter().find(|p| p.is_dir())
}

/// Lists all available switch profiles (subdirectories in the switches directory)
pub fn list_switch_profiles() -> Vec<String> {
    let mut profiles = Vec::new();
    if let Some(dir) = get_switches_dir()
        && let Ok(entries) = fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type()
                && ft.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                profiles.push(name.to_string());
            }
        }
    }
    profiles.sort();
    profiles
}

/// Returns whether the mechanical keyboard sound feature is available
pub fn is_sound_available() -> bool {
    !list_switch_profiles().is_empty()
}

pub use crate::platform::{is_windows_autostart_enabled, set_windows_autostart};
