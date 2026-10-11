//! System Tray & Icon Management for Kora UI

pub mod handler;
pub mod icon;
pub mod loader;

pub use handler::*;
pub use icon::TrayIcon;
pub use loader::{
    get_small_icon_size, is_windows_dark_taskbar, load_icon_from_memory, safe_destroy_icon,
};
